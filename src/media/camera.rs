use opencv::{
    videoio::{
        VideoCapture,
        VideoCaptureTraitConst,
        CAP_PROP_FRAME_WIDTH,
        CAP_PROP_FRAME_HEIGHT,
    },
    prelude::VideoCaptureTrait,
};

use crate::logging::LogResult;

pub struct Camera(VideoCapture);

pub struct CameraReader<'a,> {
    camera: &'a mut Camera,
    buffer: super::Frame,
}

impl Camera {
    #[inline]
    pub fn connect(source: impl CameraSource) -> LogResult<Self> {
        source.new_camera()
    }

    pub fn begin(&mut self) -> LogResult<CameraReader> {
        let w = self.0.get(CAP_PROP_FRAME_WIDTH)? as u32;
        let h = self.0.get(CAP_PROP_FRAME_HEIGHT)? as u32;

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
        Ok(Camera(VideoCapture::from_file(
            self,
            opencv::videoio::CAP_ANY)?))
    }
}

impl CameraSource for String {
    fn new_camera(&self) -> LogResult<Camera> {
        Ok(Camera(VideoCapture::from_file(
            self,
            opencv::videoio::CAP_ANY)?))
    }
}

impl CameraSource for u32 {
    fn new_camera(&self) -> LogResult<Camera> {
        Ok(Camera(VideoCapture::new(
            *self as i32,
            opencv::videoio::CAP_ANY)?))
    }
}

impl<'a, 'b> CameraReader<'a> {
    pub fn next(&'b mut self) -> Option<LogResult<&'b super::Frame>> {
        match self.camera.0.read(&mut self.buffer) {
            Ok(exists) => if exists { Some(Ok(&self.buffer)) }
            else { None },
            Err(error) => Some(Err(error.into())),
        }
    }
}
