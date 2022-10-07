use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

pub struct AppData {
    db_pool: Pool<ConnectionManager<PgConnection>>
}

impl AppData {
    pub fn create(database_url: &str) -> Self {

        let manager = ConnectionManager::new(database_url);

        Self {
            db_pool: Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .expect("Could not build connection pool")
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
}