use diesel::{PgConnection, Connection};
use tokio::sync::Mutex;

pub struct AppData {
    pub(crate) db_connection: Mutex<PgConnection>,
}

impl AppData {
    pub fn create(database_url: &str) -> Self {
        Self {
            db_connection: Mutex::new(PgConnection::establish(database_url).unwrap())
        }
    }

    #[allow(mutable_transmutes)]
    pub fn argon2(&self, password: &str) -> [u8; 64] {
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

        return hash;
    }
}