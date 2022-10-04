use diesel::{PgConnection, Connection};

pub struct AppState {
    pub(crate) db_connection: PgConnection,
}

impl AppState {
    pub fn create(database_url: &str) -> Self {
        Self {
            db_connection: PgConnection::establish(database_url).unwrap()
        }
    }
}