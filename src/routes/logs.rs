use tokio::sync::Mutex;

use actix_web::http::StatusCode;
use actix_web::{get, web};
use actix_web::Responder;

use serde::{Serialize, Deserialize};

use crate::logging::{ ResponseError, LogLevel, LogRecorder };
use crate::models::{ UserClaims, UserRole };

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LogRequest {
    pub index: i32,
    pub length: i32,
}

#[get("/entries")]
async fn get_entries((records, user, request): (web::Data<Mutex<LogRecorder>>, UserClaims, web::Json<LogRequest>)) -> super::Result<impl Responder> {
    if user.assigned_role != UserRole::SystemAdmin {
        return Err(
            ResponseError::new(
                "Non administrator trying to access logs",
                "Not accessable for non administrator",
                LogLevel::Information,
                StatusCode::UNAUTHORIZED)
        );
    }

    Ok(
        records.lock()
            .await
            .retrieve(request.index, request.length)
            .customize()
            .insert_header(("Content-Type", "application/json"))
            .with_status(StatusCode::OK)
    )
}

pub fn scope() -> actix_web::Scope {
    web::scope("/logs")
        .service(get_entries)
}
