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