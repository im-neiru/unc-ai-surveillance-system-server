use opencv::{
    videoio::{
        VideoCapture,
        VideoCaptureTraitConst,
        CAP_PROP_FRAME_WIDTH,
        CAP_PROP_FRAME_HEIGHT,
    },
    prelude::VideoCaptureTrait,
};
use tokio::sync::Mutex;

use crate::logging::LogResult;

pub struct Camera(Mutex<VideoCapture>);

pub struct CameraReader<'a,> {
    camera: &'a mut Camera,
    buffer: super::Frame,
}

impl Camera {
    #[inline]
    pub fn connect(source: impl CameraSource) -> LogResult<Self> {
        source.new_camera()
    }

    pub async fn begin(&mut self) -> LogResult<CameraReader> {
        let w; let h;

        {
            let vc = self.0.lock().await;

            w = vc.get(CAP_PROP_FRAME_WIDTH)? as u32;
            h = vc.get(CAP_PROP_FRAME_HEIGHT)? as u32;
        }

        Ok(CameraReader {
            camera: self,
            buffer: super::Frame::new(w, h)?,
        })
    }
}

pub trait CameraSource {
    fn new_camera(&self) -> crate::logging::LogResult<Camera>;
}

impl CameraSource for &str {
    fn new_camera(&self) -> crate::logging::LogResult<Camera> {
        Ok(Camera(Mutex::const_new(VideoCapture::from_file(
            self,
            opencv::videoio::CAP_ANY)?)))
    }
}

impl CameraSource for String {
    fn new_camera(&self) -> LogResult<Camera> {
        Ok(Camera(Mutex::const_new(VideoCapture::from_file(
            self,
            opencv::videoio::CAP_ANY)?)))
    }
}

impl CameraSource for u32 {
    fn new_camera(&self) -> LogResult<Camera> {
        Ok(Camera(Mutex::const_new(VideoCapture::new(
            *self as i32,
            opencv::videoio::CAP_ANY)?)))
    }
}

impl<'a, 'b> CameraReader<'a> {
    pub async fn next(&'b mut self) -> Option<LogResult<&'b super::Frame>> {
        let mut vc = self.camera.0.lock().await;

        match vc.read(&mut self.buffer) {
            Ok(exists) => if exists { Some(Ok(&self.buffer)) }
            else { None },
            Err(error) => Some(Err(error.into())),
        }
    }
}
