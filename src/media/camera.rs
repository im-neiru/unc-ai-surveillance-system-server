use opencv::{
    videoio::VideoCapture,
    prelude::VideoCaptureTrait,
    core::Mat,
};

use crate::logging::LogResult;

pub struct Camera(VideoCapture);

impl Camera {
    #[inline]
    pub fn connect(source: impl CameraSource) -> LogResult<Self> {
        source.new_camera()
    }

    pub fn frame(&mut self) -> LogResult<Mat> {
        let mut buffer = Mat::default();
        self.0.read(&mut buffer)?;

        Ok(buffer)
    }
}

pub trait CameraSource {
    fn new_camera(&self) -> crate::logging::LogResult<Camera>;
}

impl CameraSource for &str {
    fn new_camera(&self) -> crate::logging::LogResult<Camera> {
        Ok(Camera(VideoCapture::from_file(
            self,
            opencv::videoio::CAP_ANY)?))
    }
}

impl CameraSource for String {
    fn new_camera(&self) -> crate::logging::LogResult<Camera> {
        Ok(Camera(VideoCapture::from_file(
            self,
            opencv::videoio::CAP_ANY)?))
    }
}

impl CameraSource for u32 {
    fn new_camera(&self) -> crate::logging::LogResult<Camera> {
        Ok(Camera(VideoCapture::new(
            *self as i32,
            opencv::videoio::CAP_ANY)?))
    }
}
