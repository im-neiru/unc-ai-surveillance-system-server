use actix_web::http::{header::HeaderValue, StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::UserClaims;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warning,
    Information,
    Debug,
    Trace,
}

pub trait Loggable {
    fn message(&self) -> &str;
    fn level(&self) -> super::LogLevel;
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;
}

pub trait AsLoggableResponse {
    fn into_response(
        self,
        response_message: impl Into<String>,
        status_code: StatusCode,
    ) -> LoggableResponseError;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoggableError {
    message: String,
    level: super::LogLevel,
    timestamp: DateTime<Utc>,
}

impl LoggableError {
    #[allow(dead_code)]
    pub fn new(message: impl Into<String>, level: super::LogLevel) -> Self {
        Self {
            level,
            message: message.into(),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl<T: Loggable> AsLoggableResponse for T {
    fn into_response(
        self,
        response_message: impl Into<String>,
        status_code: StatusCode,
    ) -> LoggableResponseError {
        LoggableResponseError {
            log_message: self.message().to_string(),
            level: self.level(),
            response_message: response_message.into(),
            status_code,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl Loggable for LoggableError {
    #[inline]
    fn message(&self) -> &str {
        &self.message
    }

    #[inline]
    fn level(&self) -> super::LogLevel {
        self.level
    }

    #[inline]
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }
}

#[derive(Clone, Debug)]
pub struct LoggableResponseError {
    log_message: String,
    response_message: String,
    level: LogLevel,
    status_code: StatusCode,
    timestamp: DateTime<Utc>,
}

impl LoggableResponseError {
    pub fn new(
        log_message: impl Into<String>,
        response_message: impl Into<String>,
        level: LogLevel,
        status_code: StatusCode,
    ) -> Self {
        Self {
            log_message: log_message.into(),
            response_message: response_message.into(),
            level,
            status_code,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn unauthorized(user: UserClaims) -> Self {
        Self {
            log_message: format!("Denied access to user: {}", user.id),
            response_message: "User unauthorized".into(),
            level: LogLevel::Information,
            status_code: StatusCode::UNAUTHORIZED,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl Loggable for LoggableResponseError {
    #[inline]
    fn message(&self) -> &str {
        &self.log_message
    }

    #[inline]
    fn level(&self) -> super::LogLevel {
        self.level
    }

    #[inline]
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

#[derive(Deserialize, Serialize)]
struct ResponseJson<'a> {
    message: &'a str,
}

impl actix_web::ResponseError for LoggableResponseError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut res = actix_web::HttpResponse::new(self.status_code());

        res.headers_mut().insert(
            actix_web::http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        res.set_body(actix_web::body::BoxBody::new(
            serde_json::to_string(&ResponseJson {
                message: &self.response_message,
            })
            .unwrap(),
        ))
    }
}

impl std::fmt::Display for LoggableResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.log_message)
    }
}

// Conversions

/*
impl From<opencv::Error> for LoggableError {
    fn from(value: opencv::Error) -> Self {
        Self::new(
            format!("OpenCV Error: {}:{}", value.code, value.message),
            LogLevel::Trace,
        )
    }
}
*/
