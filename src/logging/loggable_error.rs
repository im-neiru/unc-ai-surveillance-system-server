use std::borrow::Borrow;

use actix_web::{Responder, CustomizeResponder, http::StatusCode};

pub struct Loggable {
    pub message: String
}

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