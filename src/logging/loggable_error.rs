use std::fmt::Display;

use actix_web::{Responder, CustomizeResponder};
use actix_web::http::StatusCode;
use actix_web::http::header::HeaderValue;
use serde_json::json;

#[derive(Clone, Debug)]
pub struct Loggable {
    pub log_message: String,
    timestamp: chrono::DateTime<chrono::Utc>
}

#[derive(Clone, Debug)]
pub struct LoggableWithResponse {
    pub log_message: Option<String>,
    pub response_message: Option<String>,
    pub status_code: actix_web::http::StatusCode,
    timestamp: chrono::DateTime<chrono::Utc>
}

#[derive(Debug, Clone)]
pub struct LogResponseError {
    pub(self) message: String,
    pub(self) status_code: actix_web::http::StatusCode,
}

impl std::fmt::Display for LogResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl actix_web::ResponseError for LogResponseError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut res = actix_web::HttpResponse::new(self.status_code());
        let body = json!({ "message": self.message }).to_string();

        res.headers_mut()
            .insert(actix_web::http::header::CONTENT_TYPE, 
                HeaderValue::from_static("application/json"));
        
        res.set_body(actix_web::body::BoxBody::new(body))
    }
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
            log_message: Some(self.log_message.clone()),
            response_message: match response_message {
                Some(msg) => Some(msg.to_owned()),
                _ => None,
            },
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
            log_message: Some(log_message.to_string()),
            response_message:  Some(response_message.to_string()),
            status_code,
            timestamp
        }
    }

    pub async fn log<const LEVEL: super::LogLevel>(&self, writer: &mut super::LogWriter<LEVEL>) -> super::LoggedResult<CustomizeResponder<String>> {
        if let Some(log_msg) =  &self.log_message {
            writer.write(log_msg, self.timestamp).await;
        }

        match self.status_code {
            code if code.is_success() => Ok(
                json!({
                    "message": self.response_message
                }).to_string()
                .customize()
                .append_header(("Content-Type", "application/json"))
                .with_status(self.status_code)
            ),
            _ => Err(LogResponseError {
                message: self.log_message
                    .clone()
                    .unwrap_or("No log message".to_owned()),
                status_code: self.status_code,
            })
        }
    }
}

impl Display for Loggable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.log_message)
    }
}

impl Display for LoggableWithResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.log_message {
            Some(msg) => msg.as_str(),
            None => "No log message",
        })
    }
}