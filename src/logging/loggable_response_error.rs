use actix_web::http::{StatusCode, header::HeaderValue};
use chrono::{DateTime, Utc};

use super::LogLevel;

#[derive(Clone, Debug)]
pub struct LoggableResponseError {
    pub(super) message: (String, String),
    pub(super) level: super::LogLevel,
    pub(super) status_code: StatusCode,
    pub(super) timestamp: DateTime<Utc>,
}

impl LoggableResponseError {
    #[inline]
    pub fn new(log_message: &str,
        response_message: &str,
        level: LogLevel,
        status_code: StatusCode) -> Self {
        Self {
            message: (log_message.to_owned(), response_message.to_owned()),
            level,
            status_code,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl super::Loggable for LoggableResponseError {   
    #[inline]
    fn message<'a>(&'a self) -> &'a str {
        self.message.0.as_str()
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

impl actix_web::ResponseError for LoggableResponseError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut res = actix_web::HttpResponse::new(self.status_code());

        res.headers_mut()
            .insert(actix_web::http::header::CONTENT_TYPE, 
                HeaderValue::from_static("application/json"));

        res.set_body(actix_web::body::BoxBody::new(
            serde_json::json!({
                "message": self.message.1
            }).to_string()
        ))
    }
}

impl std::fmt::Display for LoggableResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message.0)
    }
}