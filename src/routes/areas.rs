use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use serde::Deserialize;
use serde::Serialize;

use crate::models::AreaInsert;
use crate::models::AreaSelect;
use crate::{
    data::AppData,
    models::{UserClaims, UserRole},
};

#[derive(Deserialize)]
pub(crate) struct CreateAreaRequest {
    pub(crate) code: String,
    pub(crate) name: String,
}

#[derive(Deserialize)]
pub(crate) struct AssignRequest {
    #[serde(alias = "user-id")]
    pub(crate) user_id: uuid::Uuid,
    #[serde(alias = "area-code")]
    pub(crate) area_code: String,
}

#[derive(Serialize)]
struct CreateAreaOk {
    pub(crate) code: String,
}

#[derive(Serialize)]
struct ListAreaOk {
    pub(crate) areas: Vec<AreaSelect>,
}

#[actix_web::get("/list")]
async fn get_list(
    (state, _user): (web::Data<AppData>, UserClaims),
) -> super::Result<impl Responder> {
    use crate::schema::areas;

    let mut connection = state.connect_database();

    let area_list: Vec<AreaSelect> = areas::table.get_results(&mut connection).unwrap();

    Ok(web::Json(ListAreaOk { areas: area_list }))
}

#[actix_web::post("/create")]
async fn post_create(
    (state, request, user): (web::Data<AppData>, web::Json<CreateAreaRequest>, UserClaims),
) -> super::Result<impl Responder> {
    use crate::schema::areas::dsl::*;

    if user.assigned_role != UserRole::SecurityHead {
        return Err(crate::logging::ResponseError::unauthorized(user));
    }

    let mut connection = state.connect_database();

    let model: AreaInsert = request.0.try_into()?;

    let return_code = diesel::insert_into(areas)
        .values(&model)
        .returning(code)
        .get_result::<String>(&mut connection)
        .unwrap();

    Ok(web::Json(CreateAreaOk { code: return_code }))
}

#[actix_web::patch("/assign")]
async fn patch_assign(
    (state, request, user): (web::Data<AppData>, web::Json<AssignRequest>, UserClaims),
) -> super::Result<impl Responder> {
    use crate::schema::users::dsl::*;

    if user.assigned_role != UserRole::SecurityHead {
        return Err(crate::logging::ResponseError::unauthorized(user));
    }

    let mut connection = state.connect_database();

    diesel::update(users.filter(id.eq(request.user_id)))
        .set(assigned_area.eq(&request.area_code))
        .execute(&mut connection)
        .unwrap();

    Ok(HttpResponse::Ok())
}

pub fn scope() -> actix_web::Scope {
    web::scope("/areas")
        .service(post_create)
        .service(get_list)
        .service(patch_assign)
}
