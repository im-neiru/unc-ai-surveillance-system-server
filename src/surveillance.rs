use diesel::{
    query_dsl::{methods::FilterDsl, select_dsl::SelectDsl},
    ExpressionMethods, RunQueryDsl,
};
use opencv::{
    core::Mat,
    videoio::{VideoCapture, VideoCaptureTrait, CAP_FFMPEG},
};

use crate::data::AppData;

pub struct Surveillance {
    cameras: Vec<RtspCamera>,
}

impl Surveillance {
    pub fn new(app_data: &AppData) -> Self {
        let bytes = opencv::core::Vector::from_slice(include_bytes!("../model.onnx"));

        let mut connection = app_data.connect_database();

        use crate::schema::cameras;

        let entries: Vec<(uuid::Uuid, String, String)> = cameras::table
            .select((cameras::id, cameras::camera_url, cameras::label))
            .filter(cameras::deactivated.eq(false))
            .load(&mut connection)
            .unwrap();

        Self {
            cameras: entries
                .iter()
                .map(|(id, url, label)| RtspCamera {
                    id: *id,
                    label: label.to_owned(),
                    capture: VideoCapture::from_file(url, CAP_FFMPEG).ok(),
                    buffer: Mat::default(),
                })
                .collect(),
        }
    }
}

pub struct RtspCamera {
    id: uuid::Uuid,
    label: String,
    capture: Option<VideoCapture>,
    buffer: Mat,
}

impl RtspCamera {
    pub fn next(&mut self) -> Option<Mat> {
        if Some(true) == self.capture.as_mut()?.read(&mut self.buffer).ok() {
            return Some(self.buffer.clone());
        }

        None
    }
}
