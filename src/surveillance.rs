use actix_web::web;
use diesel::{
    query_dsl::{methods::FilterDsl, select_dsl::SelectDsl},
    ExpressionMethods, RunQueryDsl,
};
use image::{Rgb, RgbImage};
use ndarray::IxDyn;
use ndarray::{s, Array, Axis};
use opencv::core::{Point3_, CV_8UC3};
use opencv::videoio::{VideoCaptureTraitConst, CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH};
use opencv::{
    self,
    core::{Mat, Size_},
    imgproc::INTER_LINEAR,
    prelude::MatTraitConst,
    videoio::{VideoCapture, VideoCaptureTrait, CAP_FFMPEG},
};
use ort::{tensor::InputTensor, ExecutionProvider};
use std::sync::{Mutex, RwLock};

use crate::data::AppData;

pub struct Surveillance<'s> {
    session: ort::InMemorySession<'s>,
    cameras: Mutex<Vec<RtspCamera>>,
    app_data: web::Data<AppData<'s>>,
}

impl<'s> Surveillance<'s> {
    pub fn new(app_data: web::Data<AppData>) -> Self {
        let mut connection = app_data.connect_database();

        use crate::schema::cameras;

        let entries: Vec<(uuid::Uuid, String, String, String)> = cameras::table
            .select((
                cameras::id,
                cameras::camera_url,
                cameras::label,
                cameras::area_code,
            ))
            .filter(cameras::deactivated.eq(false))
            .load(&mut connection)
            .unwrap();

        Self {
            cameras: Mutex::new(
                entries
                    .iter()
                    .map(|(id, url, label, area_code)| {
                        let capture: Option<VideoCapture> =
                            VideoCapture::from_file(url, CAP_FFMPEG).ok();
                        let mut width = 640.0;
                        let mut height = 640.0;

                        if let Some(cap) = &capture {
                            width = cap.get(CAP_PROP_FRAME_WIDTH).unwrap();
                            height = cap.get(CAP_PROP_FRAME_HEIGHT).unwrap();
                        }
                        RtspCamera {
                            id: *id,
                            label: label.to_owned(),
                            area_code: area_code.to_string(),
                            capture,
                            buffer: Frame(unsafe {
                                Mat::new_size(
                                    Size_ {
                                        width: width as _,
                                        height: height as _,
                                    },
                                    CV_8UC3,
                                )
                                .unwrap()
                                .into()
                            }),
                            width: width as _,
                            height: height as _,
                        }
                    })
                    .collect(),
            ),
            session: {
                let environment = ort::Environment::builder()
                    .with_log_level(ort::LoggingLevel::Verbose)
                    .with_execution_providers([ExecutionProvider::tensorrt()])
                    .build()
                    .unwrap()
                    .into_arc();
                ort::SessionBuilder::new(&environment)
                    .unwrap()
                    .with_optimization_level(ort::GraphOptimizationLevel::Level3)
                    .unwrap()
                    .with_intra_threads(1)
                    .unwrap()
                    .with_model_from_memory(include_bytes!("../model.onnx"))
                    .unwrap()
            },
            app_data,
        }
    }

    pub async fn run(&mut self) {
        async {
            loop {
                for camera in self.cameras.lock().unwrap().iter_mut() {
                    if let Some(frame) = camera.next() {
                        frame.fit(camera.width, camera.height);
                        let predictions = self.infer(&frame);
                        for predicted_box in predictions {
                            match predicted_box.class {
                                Label::FacingBackwards => {
                                    self.app_data
                                        .store_violation(
                                            camera.area_code.clone(),
                                            crate::models::ViolationKind::FootTraffic,
                                            predicted_box.get_image(&frame),
                                        )
                                        .await
                                }
                                Label::MaskWearedIncorrect => {
                                    self.app_data
                                        .store_violation(
                                            camera.area_code.clone(),
                                            crate::models::ViolationKind::FacemaskProtocol,
                                            predicted_box.get_image(&frame),
                                        )
                                        .await
                                }
                                Label::WithoutMask => {
                                    self.app_data
                                        .store_violation(
                                            camera.area_code.clone(),
                                            crate::models::ViolationKind::FacemaskProtocol,
                                            predicted_box.get_image(&frame),
                                        )
                                        .await
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
        }
        .await
    }

    fn infer(&self, frame: &Frame) -> Vec<OutputBox> {
        let output = self.session.run([frame.as_input()]).unwrap();
        let output = output
            .get(0)
            .unwrap()
            .try_extract::<f32>()
            .unwrap()
            .view()
            .t()
            .to_owned();

        Self::process_output(output, 640, 640)
    }

    fn process_output(
        output: Array<f32, IxDyn>,
        img_width: u32,
        img_height: u32,
    ) -> Vec<OutputBox> {
        let mut boxes = Vec::new();
        let output = output.slice(s![.., .., 0]);
        for row in output.axis_iter(Axis(0)) {
            let row: Vec<_> = row.iter().copied().collect();
            let (class_id, probability) = row
                .iter()
                .skip(4)
                .enumerate()
                .map(|(index, value)| (index, *value))
                .reduce(|a, row| if row.1 > a.1 { row } else { a })
                .unwrap();
            if probability < 0.5 {
                continue;
            }
            let class = Label::from(class_id);
            let xc = row[0] / 640.0 * (img_width as f32);
            let yc = row[1] / 640.0 * (img_height as f32);
            let w = row[2] / 640.0 * (img_width as f32);
            let h = row[3] / 640.0 * (img_height as f32);
            let x1 = xc - w / 2.0;
            let x2 = xc + w / 2.0;
            let y1 = yc - h / 2.0;
            let y2 = yc + h / 2.0;
            boxes.push(OutputBox {
                x1,
                y1,
                x2,
                y2,
                class,
                probability,
                z: (x2 - x1) * (y2 - y1),
            });
        }

        boxes.sort_by(|box1, box2| box2.probability.total_cmp(&box1.probability));
        let mut result = Vec::new();
        while !boxes.is_empty() {
            let first = boxes[0];
            result.push(first);
            boxes.retain(|box1| Self::iou(&first, box1) < 0.7)
        }
        result
    }

    fn iou(box1: &OutputBox, box2: &OutputBox) -> f32 {
        Self::intersection(box1, box2) / Self::union(box1, box2)
    }

    fn union(box1: &OutputBox, box2: &OutputBox) -> f32 {
        let box1_area = (box1.x2 - box1.x1) * (box1.y2 - box1.y1);
        let box2_area = (box2.x2 - box2.x1) * (box2.y2 - box2.y1);
        box1_area + box2_area - Self::intersection(box1, box2)
    }

    fn intersection(box1: &OutputBox, box2: &OutputBox) -> f32 {
        let x1 = box1.x1.max(box2.x1);
        let y1 = box1.y1.max(box2.y1);
        let x2 = box1.x2.min(box2.x2);
        let y2 = box1.y2.min(box2.y2);
        (x2 - x1) * (y2 - y1)
    }
}

pub struct RtspCamera {
    id: uuid::Uuid,
    label: String,
    area_code: String,
    capture: Option<VideoCapture>,
    buffer: Frame,
    width: u32,
    height: u32,
}

impl RtspCamera {
    pub fn next(&mut self) -> Option<Frame> {
        if Some(true)
            == self
                .capture
                .as_mut()?
                .read(&mut self.buffer.0.get_mut().unwrap())
                .ok()
        {
            return Some(self.buffer);
        }

        None
    }
}

pub struct Frame(RwLock<Mat>);

impl Frame {
    pub(crate) fn capture(video: &mut VideoCapture, width: u32, height: u32) -> Self {
        let mut buffer: Mat = unsafe {
            Mat::new_size(
                Size_ {
                    width: width as _,
                    height: height as _,
                },
                CV_8UC3,
            )
            .unwrap()
        };
        video.read(&mut buffer).unwrap();
        Self(buffer.into())
    }

    pub(crate) fn fit(&self, frame_width: u32, frame_height: u32) -> Self {
        let min = u32::min(frame_height, frame_width);
        let x_offset = (frame_width - min) / 2;
        let y_offset = (frame_height - min) / 2;

        let cropped = Mat::roi(
            self.0.get_mut().unwrap(),
            opencv::core::Rect {
                x: x_offset as _,
                y: y_offset as _,
                width: min as _,
                height: min as _,
            },
        )
        .unwrap();

        let mut rescaled = unsafe {
            Mat::new_size(
                Size_ {
                    width: 640,
                    height: 640,
                },
                CV_8UC3,
            )
            .unwrap()
        };
        opencv::imgproc::resize(
            &cropped,
            &mut rescaled,
            Size_ {
                width: 640,
                height: 640,
            },
            0.0,
            0.0,
            INTER_LINEAR,
        )
        .unwrap();
        Self(rescaled.into())
    }

    pub(crate) fn as_input(&self) -> InputTensor {
        let mut tensor = Array::zeros((1, 3, 640, 640)).into_dyn();

        for x in 0..640 {
            for y in 0..640 {
                let element: opencv::core::Point3_<u8> =
                    *self.0.read().unwrap().at_2d(x as _, y as _).unwrap();
                tensor[[0, 0, x, y]] = (element.x as f32) / 255.0; // R
                tensor[[0, 1, x, y]] = (element.y as f32) / 255.0; // G
                tensor[[0, 2, x, y]] = (element.z as f32) / 255.0; // B
            }
        }

        InputTensor::FloatTensor(tensor)
    }
}

#[derive(Clone, Copy, Debug)]
enum Label {
    FacingBackwards,
    MaskWearedIncorrect,
    WithMask,
    WithoutMask,
    Unknown,
}

#[derive(Clone, Copy, Debug)]
struct OutputBox {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    z: f32,
    class: Label,
    probability: f32,
}

impl OutputBox {
    pub fn get_image(&self, frame: &Frame) -> RgbImage {
        let width = (self.x2 - self.x1) as _;
        let height = (self.y2 - self.y1) as _;

        RgbImage::from_fn(width, height, |x, y| {
            let pixel: Point3_<u8> = *frame
                .0
                .read()
                .unwrap()
                .at_2d(x as i32 + self.x1 as i32, y as i32 + self.y1 as i32)
                .unwrap();
            Rgb::<u8>([pixel.x, pixel.y, pixel.z])
        })
    }
}

impl From<usize> for Label {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::FacingBackwards,
            1 => Self::MaskWearedIncorrect,
            2 => Self::WithMask,
            3 => Self::WithoutMask,
            _ => Self::Unknown,
        }
    }
}

impl Label {
    const FACING_BACKWARDS: opencv::core::Scalar =
        opencv::core::Scalar::new(255.0, 255.0, 1.0, 255.0);
    const MASK_WEARED_INCORRECT: opencv::core::Scalar =
        opencv::core::Scalar::new(10.0, 200.0, 255.0, 255.0);
    const WITH_MASK: opencv::core::Scalar = opencv::core::Scalar::new(40.0, 255.0, 1.0, 255.0);
    const WITHOUT_MASK: opencv::core::Scalar = opencv::core::Scalar::new(10.0, 10.0, 255.0, 255.0);
    const UNKNOWN: opencv::core::Scalar = opencv::core::Scalar::new(200.0, 0.0, 250.0, 255.0);

    const fn color(&self) -> opencv::core::Scalar {
        match self {
            Label::FacingBackwards => Self::FACING_BACKWARDS,
            Label::MaskWearedIncorrect => Self::MASK_WEARED_INCORRECT,
            Label::WithMask => Self::WITH_MASK,
            Label::WithoutMask => Self::WITHOUT_MASK,
            Label::Unknown => Self::UNKNOWN,
        }
    }
}
