use std::fmt::Display;

use actix_web::{Responder, CustomizeResponder, http::StatusCode};

#[derive(Clone, Debug)]
pub struct Loggable {
    pub message: String
}


#[derive(Clone, Debug)]
pub struct LoggableWithResponse {
    pub message: String,
    pub status_code: actix_web::http::StatusCode,
}

impl Loggable {
    pub fn log(&self, writer: &mut super::LogWriter) {
        writer.write(&self.message);
    }

    pub fn with_response(&self, status_code: StatusCode) -> LoggableWithResponse {
        LoggableWithResponse {
            message: self.message.clone(),
            status_code }
    }
}

impl LoggableWithResponse {
    pub fn log(&self, writer: &mut super::LogWriter) -> CustomizeResponder<String> {
        writer.write(&self.message);

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