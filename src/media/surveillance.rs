use std::{
    collections::{
        HashMap,
        hash_map::DefaultHasher,
    },
    hash::{ Hash, Hasher }
};

use ring::rand::{ SystemRandom, SecureRandom };

use crate::logging::{ LogResult, LoggableResponseError };
use actix_web::http::StatusCode;

pub struct Surveillance {
    cameras: HashMap<CameraId, super::Camera, BuildCameraIdHasher>,
    rng: SystemRandom
}

impl Surveillance {
    pub fn new() -> Self {
        Self {
            cameras: HashMap::with_hasher(BuildCameraIdHasher),
            rng: SystemRandom::new()
        }
    }

    pub fn add_camera(&mut self, source: impl super::camera::CameraSource) -> LogResult<CameraId> {
        let id = CameraId::new_unique(self.cameras.keys(), &mut self.rng)?;
        self.cameras.insert(id, super::Camera::connect(source)?);

        Ok(id)
    }

    pub fn camera(&mut self, id: CameraId) -> Option<&mut super::Camera> {
        self.cameras.get_mut(&id)
    }
}

#[derive(Clone, Copy)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct CameraId(u32);

impl CameraId {
    fn new_unique<'a>(lookup: impl Iterator<Item = &'a CameraId> + Clone, rng: &mut SystemRandom) -> LogResult<CameraId> {
        let mut buffer = [0u8; 4];
        let mut id_u32;

        'outer: loop {
            if rng.fill(&mut buffer).is_err() {
                return Err(LoggableResponseError::new(
                    "Unable to generate camera ID",
                    "Camera related error",
                    crate::logging::LogLevel::Trace,
                StatusCode::INTERNAL_SERVER_ERROR));
            }

            id_u32 = u32::from_le_bytes(buffer);

            // Check for duplicates
            for recorded_id in lookup.clone() {
                if recorded_id.0 == id_u32 {
                    continue 'outer;
                }
            }

            return Ok(CameraId(id_u32));
        }
    }
}

impl Hash for CameraId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.0)
    }
}

// Optmized hasher for CameraId
// removes unecessary hashing process
struct CameraIdHasher {
    buffer: u64
}

struct BuildCameraIdHasher;

impl std::hash::BuildHasher for BuildCameraIdHasher {
    type Hasher = CameraIdHasher;
    fn build_hasher(&self) -> CameraIdHasher {
        CameraIdHasher { buffer: 0 }
    }
}

impl Hasher for CameraIdHasher {
    fn write_u8(&mut self, i: u8) {
        self.buffer = i as u64;
    }

    fn write_u16(&mut self, i: u16) {
        self.buffer = i as u64;
    }

    fn write_u32(&mut self, i: u32) {
        self.buffer = i as u64;
    }

    fn write_u64(&mut self, i: u64) {
        self.buffer = i as u64;
    }

    fn write_u128(&mut self, i: u128) {
        self.buffer = i as u64;
    }

    fn write_usize(&mut self, i: usize) {
        self.buffer = i as u64;
    }

    fn write_i8(&mut self, i: i8) {
        self.buffer = i as u64;
    }

    fn write_i16(&mut self, i: i16) {
        self.buffer = i as u64;
    }

    fn write_i32(&mut self, i: i32) {
        self.buffer = i as u64;
    }

    fn write_i64(&mut self, i: i64) {
        self.buffer = i as u64;
    }

    fn write_i128(&mut self, i: i128) {
        self.buffer = i as u64;
    }

    fn write_isize(&mut self, i: isize) {
        self.buffer = i as u64;
    }

    fn finish(&self) -> u64 {
        self.buffer
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        self.buffer = hasher.finish();
    }
}
