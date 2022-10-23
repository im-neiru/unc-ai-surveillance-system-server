use std::fmt::Display;

use actix_web::{Responder, CustomizeResponder, http::StatusCode};
use serde_json::json;

#[derive(Clone, Debug)]
pub struct Loggable {
    pub log_message: String,
    timestamp: chrono::DateTime<chrono::Utc>
}

#[derive(Clone, Debug)]
pub struct LoggableWithResponse {
    pub log_message: String,
    pub response_message: String,
    pub status_code: actix_web::http::StatusCode,
    timestamp: chrono::DateTime<chrono::Utc>
}

impl Loggable {
    
    #[inline]
    pub fn new(log_message: &str, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            log_message: log_message.to_string(),
            timestamp
        }
    }

    pub async fn log<const LEVEL: super::LogLevel>(&self, writer: &mut super::LogWriter<LEVEL>) {
        writer.write(&self.log_message, self.timestamp).await;
    }

    pub fn with_response(&self, status_code: StatusCode, response_message: Option<&str>) -> LoggableWithResponse {
        LoggableWithResponse {
            log_message: self.log_message.clone(),
            response_message: response_message.unwrap_or(&self.log_message).to_string(),
            timestamp: self.timestamp,
            status_code
        }
    }
}

impl LoggableWithResponse {

    #[inline]
    pub fn new(log_message: &str,
        response_message: &str,
        status_code: actix_web::http::StatusCode,
        timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            log_message: log_message.to_string(),
            response_message: response_message.to_string(),
            status_code,
            timestamp
        }
    }

    pub async fn log<const LEVEL: super::LogLevel>(&self, writer: &mut super::LogWriter<LEVEL>) -> CustomizeResponder<String> {
        writer.write(&self.log_message, self.timestamp).await;

        json!({ "message": self.response_message })
            .to_string()
            .customize()
            .with_status(self.status_code)
    }
}

impl Display for Loggable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.log_message)
    }
}

impl Display for LoggableWithResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.log_message)
    }
}