use std::fmt::Display;

use actix_web::{Responder, CustomizeResponder, http::StatusCode};

#[derive(Clone, Debug)]
pub struct Loggable {
    pub message: String,
    timestamp: chrono::DateTime<chrono::Utc>
}


#[derive(Clone, Debug)]
pub struct LoggableWithResponse {
    pub message: String,
    pub status_code: actix_web::http::StatusCode,
    timestamp: chrono::DateTime<chrono::Utc>
}

impl Loggable {
    pub async fn log<const LEVEL: super::LogLevel>(&self, writer: &mut super::LogWriter<LEVEL>) {
        writer.write(&self.message, self.timestamp).await;
    }

    pub fn with_response(&self, status_code: StatusCode) -> LoggableWithResponse {
        LoggableWithResponse {
            message: self.message.clone(),
            timestamp: self.timestamp,
            status_code
        }
    }
}

impl LoggableWithResponse {
    pub async fn log<const LEVEL: super::LogLevel>(&self, writer: &mut super::LogWriter<LEVEL>) -> CustomizeResponder<String> {
        writer.write(&self.message, self.timestamp).await;

        self.message
            .clone()
            .customize()
            .with_status(self.status_code)
    }
}

impl Display for Loggable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Display for LoggableWithResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}