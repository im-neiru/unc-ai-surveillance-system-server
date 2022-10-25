use actix_web::http::StatusCode;
use chrono::{DateTime, Utc};

pub struct LoggableResponseError {
    pub(super) message: (String, String),
    pub(super) level: super::LogLevel,
    pub(super) status_code: StatusCode,
    pub(super) timestamp: DateTime<Utc>,
}