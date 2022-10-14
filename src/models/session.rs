use diesel::Insertable;
use super::DeviceOs;

use chrono::NaiveDateTime;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::sessions)]
pub struct SessionInsert {
    user_id: uuid::Uuid,
    created_time: NaiveDateTime,
    last_login: NaiveDateTime,
    logout_time: Option<NaiveDateTime>,
    device_os: DeviceOs,
    device_name: String,
    device_hash: Vec<u8>
}

impl SessionInsert {
    pub fn create(user_id: &uuid::Uuid,
                    device_os: &DeviceOs,
                    device_name: &str,
                    device_hash: &[u8]) -> Self {
        let now = chrono::Utc::now().naive_utc();

        Self {
            user_id: user_id.to_owned(),
            created_time: now,
            last_login: now,
            logout_time: None,
            device_os: device_os.to_owned(),
            device_name: device_name.to_owned(),
            device_hash: device_hash.to_vec(),
        }
    }
}