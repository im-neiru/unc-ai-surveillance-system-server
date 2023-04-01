use crate::logging::LogLevel;
use crate::models::ViolationUnknown;
use crate::{data::AppData, models::UserClaims};
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use serde::Deserialize;

#[actix_web::get("/unidentified")]
async fn get_unidentified(
    (state, _user): (web::Data<AppData>, UserClaims),
) -> super::Result<impl Responder> {
    use crate::schema::violations::dsl::*;

    let mut connection = state.connect_database();

    let list = violations
        .filter(identified.eq(false))
        .select((id, area_code, violation_kind, date_time))
        .get_results::<ViolationUnknown>(&mut connection)
        .unwrap();

    Ok(serde_json::to_string(&list)
        .unwrap()
        .customize()
        .with_status(StatusCode::OK))
}

#[derive(Deserialize)]
struct GetImageQuery {
    id: uuid::Uuid,
}

#[actix_web::get("/image")]
async fn get_image(
    (state, query, _user): (web::Data<AppData>, web::Query<GetImageQuery>, UserClaims),
) -> super::Result<impl Responder> {
    use crate::schema::violations::dsl::*;

    let mut connection = state.connect_database();

    let image = violations
        .filter(id.eq(query.id))
        .select(image_bytes)
        .first::<Vec<u8>>(&mut connection)
        .or(Err(crate::logging::ResponseError::new(
            "Failed to find image",
            "Image not found",
            LogLevel::Information,
            StatusCode::NOT_FOUND,
        )))?;

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("image/jpeg")
        .body(image))
}

pub fn scope() -> actix_web::Scope {
    web::scope("/users")
        .service(get_unidentified)
        .service(get_image)
}
