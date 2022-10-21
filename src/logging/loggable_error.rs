pub struct Loggable {
    pub message: String
}

pub struct LoggableWithResponse {
    pub message: String,
    pub status_code: actix_web::http::StatusCode,
}