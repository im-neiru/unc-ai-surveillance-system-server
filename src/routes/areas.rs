use actix_web::http::StatusCode;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use diesel::dsl::count;
use diesel::result::DatabaseErrorKind;
use diesel::ExpressionMethods;
use diesel::JoinOnDsl;
use diesel::NullableExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use serde::Deserialize;
use serde::Serialize;

use crate::logging::LogLevel;
use crate::models::AreaGuardCount;
use crate::models::AreaInsert;
use crate::models::AreaSelect;
use crate::models::CameraInsert;
use crate::models::IntoModel;

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
    pub(crate) area_code: Option<String>,
}
#[derive(Deserialize)]
struct AreaRemoveQuery {
    #[serde(alias = "area-code")]
    pub(crate) code: String,
}

#[derive(Serialize)]
struct CreateAreaOk {
    pub(crate) code: String,
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    #[serde(alias = "count-guards")]
    count_guards: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct CameraRemoveQuery {
    #[serde(alias = "id")]
    camera_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
struct CameraModifyRequest {
    #[serde(alias = "id")]
    camera_id: uuid::Uuid,
    label: Option<String>,
    #[serde(alias = "camera-url")]
    camera_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CameraAddRequest {
    label: String,
    #[serde(alias = "area-code")]
    area_code: String,
    #[serde(alias = "camera-url")]
    camera_url: String,
    enable: bool,
}

impl IntoModel<CameraInsert> for CameraAddRequest {
    fn model(&self) -> crate::routes::Result<CameraInsert> {
        crate::logging::ResponseError::length_limit_check("Label", &self.label, 3, 15)?;
        crate::logging::ResponseError::length_limit_check("Area code", &self.area_code, 3, 10)?;
        crate::logging::ResponseError::length_limit_check("Camera URL", &self.camera_url, 10, 512)?;

        Ok(CameraInsert {
            label: self.label.clone(),
            area_code: self.area_code.clone(),
            camera_url: self.camera_url.clone(),
            deactivated: !self.enable,
        })
    }
}

#[actix_web::get("/list")]
async fn get_list(
    (state, _user, query): (web::Data<AppData<'_>>, UserClaims, web::Query<ListQuery>),
) -> super::Result<impl Responder> {
    use crate::schema::areas;
    use crate::schema::users;

    let mut connection = state.connect_database();

    if query.count_guards == Some(true) {
        let list = areas::table
            .left_join(users::table.on(areas::code.nullable().eq(users::assigned_area)))
            .filter(users::deactivated.eq(false))
            .group_by((areas::code, areas::name))
            .select((
                areas::dsl::code,
                areas::dsl::name,
                count(users::dsl::assigned_area.assume_not_null()),
            ))
            .load::<AreaGuardCount>(&mut connection)
            .unwrap();
        Ok(serde_json::to_string(&list)
            .unwrap()
            .customize()
            .with_status(StatusCode::OK))
    } else {
        let list: Vec<AreaSelect> = areas::table.get_results(&mut connection).unwrap();
        Ok(serde_json::to_string(&list)
            .unwrap()
            .customize()
            .with_status(StatusCode::OK))
    }
}

#[actix_web::post("/create")]
async fn post_create(
    (state, request, user): (web::Data<AppData<'_>>, web::Json<CreateAreaRequest>, UserClaims),
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

#[actix_web::delete("/remove")]
async fn delete_areas(
    (state, query, user): (web::Data<AppData<'_>>, web::Query<AreaRemoveQuery>, UserClaims),
) -> super::Result<impl Responder> {
    use crate::schema::areas;

    if user.assigned_role != UserRole::SecurityHead {
        return Err(crate::logging::ResponseError::unauthorized(user));
    }
    let mut connection = state.connect_database();

    diesel::delete(areas::table.filter(areas::code.eq(&query.code)))
        .execute(&mut connection)
        .unwrap();

    Ok(HttpResponse::NoContent())
}

#[actix_web::patch("/assign")]
async fn patch_assign(
    (state, request, user): (web::Data<AppData<'_>>, web::Json<AssignRequest>, UserClaims),
) -> super::Result<impl Responder> {
    use crate::schema::users::dsl::*;

    if user.assigned_role != UserRole::SecurityHead {
        return Err(crate::logging::ResponseError::unauthorized(user));
    }
    let mut connection = state.connect_database();

    let role: UserRole = users
        .filter(id.eq(request.user_id))
        .filter(deactivated.eq(false))
        .select(assigned_role)
        .get_result(&mut connection)
        .or(Err(crate::logging::ResponseError::new(
            "Failed to find user",
            "User does not exists",
            LogLevel::Information,
            StatusCode::NOT_ACCEPTABLE,
        )))?;

    if role != UserRole::SecurityGuard {
        return Err(crate::logging::ResponseError::new(
            "Invalid user role",
            "User must be a security guard",
            LogLevel::Information,
            StatusCode::NOT_ACCEPTABLE,
        ));
    }

    diesel::update(users.filter(id.eq(request.user_id)))
        .set(assigned_area.eq(&request.area_code))
        .execute(&mut connection)
        .or(Err(crate::logging::ResponseError::new(
            "Failed to assign user",
            "Failed to assign user",
            LogLevel::Information,
            StatusCode::NOT_ACCEPTABLE,
        )))?;

    Ok(HttpResponse::Ok())
}

#[actix_web::post("/camera")]
async fn post_camera(
    (state, request, user): (web::Data<AppData<'_>>, web::Json<CameraAddRequest>, UserClaims),
) -> super::Result<impl Responder> {
    use crate::schema::cameras;

    if user.assigned_role == UserRole::SecurityGuard {
        return Err(crate::logging::ResponseError::unauthorized(user));
    }

    let mut connection = state.connect_database();

    match diesel::insert_into(cameras::table)
        .values(request.model()?)
        .returning(cameras::id)
        .get_result::<uuid::Uuid>(&mut connection)
    {
        Err(diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            Err(crate::logging::ResponseError::conflict_field("Name"))
        }
        Err(diesel::result::Error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _)) => {
            Err(crate::logging::ResponseError::value_do_not_exist(
                &request.area_code,
            ))
        }
        Err(_) => Err(crate::logging::ResponseError::server_error()),
        Ok(id) => Ok(serde_json::json!({ "id": id })
            .to_string()
            .customize()
            .with_status(StatusCode::OK)),
    }
}

#[actix_web::patch("/camera")]
async fn patch_camera(
    (state, request, user): (
        web::Data<AppData<'_>>,
        web::Json<CameraModifyRequest>,
        UserClaims,
    ),
) -> super::Result<impl Responder> {
    use crate::schema::cameras;

    if user.assigned_role == UserRole::SecurityGuard {
        return Err(crate::logging::ResponseError::unauthorized(user));
    }

    let mut connection = state.connect_database();

    let result = if let (Some(label), Some(camera_url)) = (&request.label, &request.camera_url) {
        diesel::update(cameras::table.filter(cameras::id.eq(request.camera_id)))
            .set((cameras::label.eq(label), cameras::camera_url.eq(camera_url)))
            .execute(&mut connection)
    } else if let Some(label) = &request.label {
        diesel::update(cameras::table.filter(cameras::id.eq(request.camera_id)))
            .set(cameras::label.eq(label))
            .execute(&mut connection)
    } else if let Some(camera_url) = &request.camera_url {
        diesel::update(cameras::table.filter(cameras::id.eq(request.camera_id)))
            .set(cameras::camera_url.eq(camera_url))
            .execute(&mut connection)
    } else {
        return Err(crate::logging::ResponseError::new(
            "Nothing to do",
            "Nothing to do",
            LogLevel::Information,
            StatusCode::NOT_ACCEPTABLE,
        ));
    };

    match result {
        Err(_) => Err(crate::logging::ResponseError::server_error()),
        Ok(row_count) => {
            if row_count == 0 {
                Err(crate::logging::ResponseError::value_do_not_exist("Camera"))
            } else {
                Ok(HttpResponse::NoContent())
            }
        }
    }
}

#[actix_web::delete("/camera")]
async fn delete_camera(
    (state, query, user): (
        web::Data<AppData<'_>>,
        web::Query<CameraRemoveQuery>,
        UserClaims,
    ),
) -> super::Result<impl Responder> {
    use crate::schema::cameras;

    if user.assigned_role == UserRole::SecurityGuard {
        return Err(crate::logging::ResponseError::unauthorized(user));
    }

    let mut connection = state.connect_database();

    match diesel::delete(cameras::table.filter(cameras::id.eq(query.camera_id)))
        .execute(&mut connection)
    {
        Err(_) => Err(crate::logging::ResponseError::server_error()),
        Ok(row_count) => {
            if row_count == 0 {
                Err(crate::logging::ResponseError::value_do_not_exist("Camera"))
            } else {
                Ok(HttpResponse::NoContent())
            }
        }
    }
}

pub fn scope() -> actix_web::Scope {
    web::scope("/areas")
        .service(post_create)
        .service(get_list)
        .service(patch_assign)
        .service(delete_areas)
        .service(post_camera)
        .service(patch_camera)
        .service(delete_camera)
}
