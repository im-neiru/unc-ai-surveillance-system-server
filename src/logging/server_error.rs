use actix_web::http::{StatusCode, header::HeaderValue};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct ServerError {
    pub(super) message: String,
    pub(super) level: super::LogLevel,
    pub(super) timestamp: DateTime<Utc>,
}

impl super::Loggable for ServerError {   
    #[inline]
    fn message<'a>(&'a self) -> &'a str {
        &self.message
    }

    fn level(&self) -> super::LogLevel {
        self.level
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

impl actix_web::ResponseError for ServerError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut res = actix_web::HttpResponse::new(self.status_code());

        res.headers_mut()
            .insert(actix_web::http::header::CONTENT_TYPE, 
                HeaderValue::from_static("application/json"));

        res.set_body(actix_web::body::BoxBody::new(
            "{\"message\": \"Server fatal error\"}"
        ))
    }
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}