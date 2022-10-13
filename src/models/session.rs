use diesel::Insertable;
use super::DeviceOs;

use chrono::NaiveDateTime;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::sessions)]
pub struct SessionInsert<'a> {
    user_id: uuid::Uuid,
    created_time: NaiveDateTime,
    last_login: NaiveDateTime,
    logout_time: Option<NaiveDateTime>,
    device_os: DeviceOs,
    device_name: &'a str,
    device_hash: &'a [u8]
}

impl SessionInsert<'_> {
    pub fn create(user_id: uuid::Uuid,
                    device_os: DeviceOs,
                    device_name: &str,
                    device_hash: &[u8]) -> Self {
        let now = chrono::Utc::now().naive_utc();

        Self {
            user_id,
            created_time: now,
            last_login: now,
            logout_time: None,
            device_os,
            device_name,
            device_hash,
        }
    }
}