use crate::logging::LogLevel;
use crate::models::{Category, IdentifiedViolation, UserRole, ViolationUnknown};
use crate::{data::AppData, models::UserClaims};
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder};
use diesel::BoolExpressionMethods;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use serde::Deserialize;

#[actix_web::get("/unidentified")]
async fn get_unidentified(
    (state, user): (web::Data<AppData<'_>>, UserClaims),
) -> super::Result<impl Responder> {
    use crate::schema::users;
    use crate::schema::violations::dsl::*;

    let mut connection = state.connect_database();
    let mut list = Vec::new();

    if user.assigned_role == UserRole::SecurityGuard {
        let assigned_area: Option<String> = users::table
            .filter(users::id.eq(user.id))
            .select(users::assigned_area)
            .first(&mut connection)
            .unwrap();

        if let Some(area) = assigned_area {
            list = violations
                .filter(identified.eq(false).and(area_code.eq(area)))
                .order_by(date_time)
                .select((id, area_code, violation_kind, date_time))
                .get_results::<ViolationUnknown>(&mut connection)
                .unwrap();
        }
    } else {
        list = violations
            .filter(identified.eq(false))
            .order_by(date_time)
            .select((id, area_code, violation_kind, date_time))
            .get_results::<ViolationUnknown>(&mut connection)
            .unwrap();
    }

    Ok(serde_json::to_string(&list)
        .unwrap()
        .customize()
        .with_status(StatusCode::OK))
}

#[derive(Deserialize)]
struct GetIdentifiedQuery {
    #[serde(alias = "area-code")]
    area_code: Option<String>,
}

#[actix_web::get("/identified")]
async fn get_identified(
    (state, user, query): (
        web::Data<AppData<'_>>,
        UserClaims,
        web::Query<GetIdentifiedQuery>,
    ),
) -> super::Result<impl Responder> {
    use crate::schema::violations::dsl::*;

    let mut connection = state.connect_database();

    if user.assigned_role == UserRole::SystemAdmin {
        return Err(crate::logging::ResponseError::unauthorized(user));
    }

    let list = match query.area_code.clone() {
        Some(area) => violations
            .filter(identified.eq(true))
            .filter(area_code.eq(area))
            .order_by(date_time)
            .select((
                id,
                area_code,
                violation_kind,
                date_time,
                personnel_id,
                first_name,
                last_name,
                category,
            ))
            .get_results::<IdentifiedViolation>(&mut connection)
            .unwrap(),
        None => violations
            .filter(identified.eq(true))
            .order_by(date_time)
            .select((
                id,
                area_code,
                violation_kind,
                date_time,
                personnel_id,
                first_name,
                last_name,
                category,
            ))
            .get_results::<IdentifiedViolation>(&mut connection)
            .unwrap(),
    };

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
    (state, query, _user): (web::Data<AppData<'_>>, web::Query<GetImageQuery>, UserClaims),
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

#[derive(Deserialize)]
struct PatchRecordRequest {
    #[serde(alias = "violation-id")]
    id: uuid::Uuid,
    #[serde(alias = "first-name")]
    first_name: String,
    #[serde(alias = "last-name")]
    last_name: String,
    #[serde(alias = "category")]
    category: Category,
}

#[actix_web::patch("/record")]
async fn patch_record(
    (state, request, user): (
        web::Data<AppData<'_>>,
        web::Json<PatchRecordRequest>,
        UserClaims,
    ),
) -> super::Result<impl Responder> {
    use crate::schema::violations;

    if user.assigned_role == UserRole::SystemAdmin {
        return Err(crate::logging::ResponseError::unauthorized(user));
    }

    let mut connection = state.connect_database();

    diesel::update(violations::table.filter(violations::id.eq(request.id)))
        .set((
            violations::personnel_id.eq(Some(user.id)),
            violations::first_name.eq(Some(&request.first_name)),
            violations::last_name.eq(Some(&request.last_name)),
            violations::category.eq(Some(&request.category)),
            violations::identified.eq(true),
        ))
        .execute(&mut connection)
        .unwrap();

    Ok(HttpResponse::build(StatusCode::OK))
}

pub fn scope() -> actix_web::Scope {
    web::scope("/violations")
        .service(get_unidentified)
        .service(get_identified)
        .service(get_image)
        .service(patch_record)
}
