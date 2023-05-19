use diesel::RunQueryDsl;
use std::io::{Cursor, Read, Seek};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use image::ImageOutputFormat;
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut, text_size};
use imageproc::rect::Rect;
use rusttype::{Font, Scale};

use actix_web::http::StatusCode;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::PgConnection;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use tokio::sync::Mutex;
use xxhash_rust::xxh3::Xxh3;

use crate::logging::{LogLevel, ResponseError};
use crate::models::{JwtClaims, PasswordHash, ViolationKind, ViolationUnknownInsert};
use crate::notifier::{Notification, Notifier};

pub struct AppData<'a> {
    db_pool: Pool<ConnectionManager<PgConnection>>,
    xxh3: Mutex<Xxh3>,
    font: Font<'a>,
    notifier: RwLock<Notifier>,
}

impl<'a> AppData<'a> {
    const JWT_SECRET: &'static str = "2b9e6f9ec298c3a7ebde69e941ed2d81";

    pub fn create(database_url: &str) -> Self {
        let manager = ConnectionManager::new(database_url);

        Self {
            db_pool: Pool::builder()
                .test_on_check_out(true)
                .build(manager)
                .expect("Could not build connection pool"),
            xxh3: Mutex::new(Xxh3::with_seed(0x13ac0750331f23db)),
            font: {
                Font::try_from_vec(Vec::from(
                    include_bytes!("../../assets/Roboto-Medium.ttf") as &[u8]
                ))
                .unwrap()
            },
            notifier: Notifier::default().into(),
        }
    }

    #[inline(always)]
    pub async fn notifier(&self) -> RwLockReadGuard<'_, Notifier> {
        self.notifier.read().await
    }

    #[inline(always)]
    pub async fn notifier_mut(&self) -> RwLockWriteGuard<'_, Notifier> {
        self.notifier.write().await
    }

    pub fn connect_database(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.db_pool.get().unwrap()
    }

    #[allow(mutable_transmutes)]
    #[allow(invalid_value)]
    #[allow(clippy::uninit_assumed_init)]
    pub fn argon2(&self, password: &str) -> crate::models::PasswordHash {
        use std::mem::*;

        let mut salt: [u8; 16] = *b"salty#Q9YNePSTpw";
        let mut hash: [u8; 64] = unsafe { MaybeUninit::uninit().assume_init() };

        let mut context = argon2::Context {
            out: &mut hash,
            pwd: Some(unsafe { transmute::<&[u8], &mut [u8]>(password.as_bytes()) }),
            salt: Some(&mut salt),
            secret: None,
            ad: None,
            t_cost: 8,
            m_cost: 1024,
            lanes: 4,
            threads: 4,
            version: argon2::Version::Version13,
            flags: argon2::Flags::DEFAULT,
        };

        argon2::i_ctx(&mut context).unwrap();

        hash.into()
    }

    pub async fn validate_password(
        &self,
        hash: PasswordHash,
        password: &str,
    ) -> crate::Result<(), ResponseError> {
        if hash == self.argon2(password) {
            return Ok(());
        }

        Err(ResponseError::new(
            "A user entered invalid password",
            "Invalid username or password",
            LogLevel::Information,
            StatusCode::UNAUTHORIZED,
        ))
    }

    pub fn jwt_encode(&self, claims: &JwtClaims) -> crate::Result<String, ResponseError> {
        match jsonwebtoken::encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(Self::JWT_SECRET.as_ref()),
        ) {
            Ok(jwt) => Ok(jwt),
            Err(err) => Err(ResponseError::new(
                "Encoding JSON Web token failed",
                match err.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Session expired",
                    _ => "Invalid Session",
                },
                LogLevel::Information,
                StatusCode::UNAUTHORIZED,
            )),
        }
    }

    pub fn jwt_decode(&self, jwt: &str) -> crate::Result<JwtClaims, ResponseError> {
        match jsonwebtoken::decode(
            jwt,
            &DecodingKey::from_secret(Self::JWT_SECRET.as_ref()),
            &Validation::default(),
        ) {
            Ok(data) => Ok(data.claims),
            Err(err) => Err(ResponseError::new(
                "Decoding JSON Web token failed",
                match err.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Session expired",
                    _ => "Invalid Session",
                },
                LogLevel::Information,
                StatusCode::UNAUTHORIZED,
            )),
        }
    }

    pub async fn xxh3_128bits<const N: usize>(&self, data: [u8; N]) -> u128 {
        let mut xxh3 = self.xxh3.lock().await;

        xxh3.update(&data);
        xxh3.digest128()
    }

    pub async fn store_violation(
        &self,
        area_code: String,
        violation_kind: ViolationKind,
        image: image::RgbImage,
    ) {
        use crate::schema::violations;

        let mut connection = self.connect_database();

        let mut image_buffer = Vec::<u8>::new();

        {
            let mut stream = Cursor::new(Vec::<u8>::new());
            image
                .write_to(&mut stream, ImageOutputFormat::Jpeg(100))
                .unwrap();
            stream.rewind().unwrap();
            stream.read_to_end(&mut image_buffer).unwrap();
        }

        let violation: uuid::Uuid = diesel::insert_into(violations::table)
            .values(ViolationUnknownInsert {
                area_code,
                violation_kind,
                date_time: chrono::Utc::now().naive_utc(),
                image_bytes: image_buffer,
                identified: false,
            })
            .returning(violations::id)
            .get_result(&mut connection)
            .unwrap();

        self.notifier()
            .await
            .notify(Notification::NewViolations(vec![violation]))
    }

    fn random_color() -> Rgb<u8> {
        let h = fastrand::f32();

        let h_i = (h * 6.0).floor() as i32;
        let f = h * 6.0 - h_i as f32;
        let p = 0.44;
        let q = (1.0 - f * 0.4) * 0.8;
        let t = (1.0 - (1.0 - f) * 0.45) * 0.8;

        let (r, g, b) = match h_i % 6 {
            0 => (0.8, t, p),
            1 => (q, 0.8, p),
            2 => (p, 0.8, t),
            3 => (p, q, 0.8),
            4 => (t, p, 0.8),
            5 => (1.0, p, q),
            _ => (0.0, 0.0, 0.0),
        };

        Rgb([
            (r * 255.0 + 0.5) as u8,
            (g * 255.0 + 0.5) as u8,
            (b * 255.0 + 0.5) as u8,
        ])
    }

    pub fn draw_default_avatar(&self, first_name: &str) -> crate::routes::Result<Vec<u8>> {
        let mut image = RgbImage::new(256, 256);

        let scale = Scale { x: 166.0, y: 166.0 };

        let text = &first_name[0..1];

        let (tw, th) = text_size(scale, &self.font, text);
        draw_filled_rect_mut(
            &mut image,
            Rect::at(0, 0).of_size(256, 256),
            Self::random_color(),
        );
        draw_text_mut(
            &mut image,
            Rgb([250u8, 250u8, 250u8]),
            (246 - tw) / 2i32,
            (240 - th) / 2i32,
            scale,
            &self.font,
            text,
        );

        let mut image_bytes = Vec::new();

        {
            let mut stream = Cursor::new(Vec::<u8>::new());
            image
                .write_to(&mut stream, ImageOutputFormat::Jpeg(100))
                .or(Err(crate::logging::ResponseError::server_error()))?;
            stream
                .rewind()
                .or(Err(crate::logging::ResponseError::server_error()))?;
            stream
                .read_to_end(&mut image_bytes)
                .or(Err(crate::logging::ResponseError::server_error()))?;
        }

        Ok(image_bytes)
    }
}
