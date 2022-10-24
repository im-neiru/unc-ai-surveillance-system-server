use actix_web::http::StatusCode;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use jsonwebtoken::{Header, EncodingKey, Validation, DecodingKey};
use tokio::sync::Mutex;
use xxhash_rust::xxh3::Xxh3;

use crate::logging::LoggableWithResponse;
use crate::models::{JwtClaims, PasswordHash};

pub struct AppData {
    db_pool: Pool<ConnectionManager<PgConnection>>,
    xxh3: Mutex<Xxh3>
}

impl AppData {
    const JWT_SECRET: &str = "2b9e6f9ec298c3a7ebde69e941ed2d81";
    
    pub fn create(database_url: &str) -> Self {

        let manager = ConnectionManager::new(database_url);

        Self {
            db_pool: Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .expect("Could not build connection pool"),
            xxh3: Mutex::new(Xxh3::with_seed(0x13ac0750331f23db))
        }
    }

    pub fn connect_database(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.db_pool.get().unwrap()
    }

    #[allow(mutable_transmutes)]
    pub fn argon2(&self, password: &str) -> crate::models::PasswordHash {
        use std::mem::*;

        let mut salt: [u8; 16] = *b"salty#Q9YNePSTpw";
        let mut hash : [u8; 64] =  unsafe { MaybeUninit::uninit().assume_init() };

        let mut context = argon2::Context {
            out:        &mut hash,
            pwd:        Some(unsafe { transmute::<&[u8], &mut [u8]>(password.as_bytes()) }),
            salt:       Some(&mut salt),
            secret:     None,
            ad:         None,
            t_cost:     8,
            m_cost:     1024,
            lanes:      4,
            threads:    4,
            version:    argon2::Version::Version13,
            flags:      argon2::Flags::DEFAULT,
        };

        argon2::i_ctx(&mut context).unwrap();

        return hash.into();
    }

    pub async fn validate_password(&self, hash: PasswordHash, password: &str) -> Result<(), LoggableWithResponse> {
        if hash == self.argon2(password) {
            return Ok(());
        }

        Err(LoggableWithResponse::new(
            "A user has entered invalid password",
            "Invalid username or password",
            StatusCode::UNAUTHORIZED))
    }

    pub fn jwt_encode(&self, claims: &JwtClaims) -> String {
        jsonwebtoken::encode(&Header::default(),
            claims, 
            &EncodingKey::from_secret(Self::JWT_SECRET.as_ref()))
            .unwrap()
    }

    pub fn jwt_decode(&self, jwt: &str) -> JwtClaims {
        jsonwebtoken::decode(jwt, 
            &DecodingKey::from_secret(Self::JWT_SECRET.as_ref()), 
            &Validation::default())
            .unwrap()
            .claims
    }

    pub async fn xxh3_128bits<const N: usize>(&self, data: [u8; N]) -> u128 {
        let mut xxh3  = self.xxh3.lock().await;

        xxh3.update(&data);
        xxh3.digest128()
    }
}