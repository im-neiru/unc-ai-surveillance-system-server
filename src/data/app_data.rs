use diesel::{PgConnection, Connection};

pub struct AppData<'a> {
    pub(crate) db_connection: PgConnection,
    argon_config: argon2::Config<'a>,
}

impl AppData<'_> {
    const SALT: &[u8] = b"salty#Q9YNePSTpw";

    pub fn create(database_url: &str) -> Self {
        use argon2::*;

        let config = Config {
            variant: Variant::Argon2i,
            version: Version::Version13,
            mem_cost: 4096,
            time_cost: 4,
            lanes: 4,
            thread_mode: ThreadMode::Parallel,
            secret: &[],
            ad: &[],
            hash_length: 64
        };

        Self {
            db_connection: PgConnection::establish(database_url).unwrap(),
            argon_config: config
        }
    }

    pub fn password_hash(&self, password: &str) -> String {
        argon2::hash_encoded(password.as_bytes(), Self::SALT, &self.argon_config).unwrap()
    }
}