use std::cmp::Ordering;
use std::io::{Cursor, Read, Seek, Write};

use actix_web::http::StatusCode;
use actix_web::{post, web};
use actix_web::{HttpResponse, Responder};

use chrono::Utc;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::result::DatabaseErrorKind;
use diesel::BoolExpressionMethods;
use diesel::{ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl};
use futures_util::StreamExt;
use image::ImageOutputFormat;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::data::AppData;
use crate::logging::{LogLevel, ResponseError};
use crate::models::{
    DeviceOs, DeviceSignature, GuardSelect, JwtClaims, SessionInsert, UserBasicSelect, UserClaims,
    UserInsert, UserRole, UserSelect,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(alias = "device-os")]
    pub device_os: DeviceOs,
    #[serde(alias = "device-name")]
    pub device_name: String,
    #[serde(alias = "device-signature")]
    pub device_signature: DeviceSignature,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct CreateUserOk {
    pub id: uuid::Uuid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct CreateUserRequest {
    username: String,
    #[serde(alias = "first-name")]
    first_name: String,
    #[serde(alias = "last-name")]
    last_name: String,
    password: String,
    #[serde(alias = "assigned-role")]
    assigned_role: UserRole,
}

impl CreateUserRequest {
    fn model(&self, state: web::Data<AppData<'_>>) -> UserInsert {
        UserInsert {
            username: self.username.clone(),
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            password_hash: state.argon2(&self.password),
            deactivated: false,
            assigned_role: self.assigned_role,
            assigned_area: None,
        }
    }
}

#[derive(Deserialize)]
struct GetAvatarQuery {
    id: Option<uuid::Uuid>,
}

enum Filter {
    Assigned = 0,
    Unassigned = 1,
}

#[derive(Deserialize)]
struct GuardQuery {
    filter: Option<Filter>,
}

impl<'de> Deserialize<'de> for Filter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        match value {
            0 => Ok(Self::Assigned),
            1 => Ok(Self::Unassigned),
            _ => Err(serde::de::Error::custom(format!(
                "invalid value for filter: {value}"
            ))),
        }
    }
}

#[derive(Serialize)]
struct GuardResponse {
    id: uuid::Uuid,
    #[serde(rename = "last-name")]
    last_name: String,
    #[serde(rename = "first-name")]
    first_name: String,
    #[serde(rename = "area-code")]
    area_code: Option<String>,
}

#[post("/login")]
async fn post_login(
    (body, state): (web::Json<LoginRequest>, web::Data<AppData<'_>>),
) -> super::Result<impl Responder> {
    let mut database = state.connect_database();
    let user = UserSelect::select_by_username(&mut database, &body.username)?;

    state
        .validate_password(user.password_hash, &body.password)
        .await?;
    let jwt = create_session(state, &mut database, &body, user).await?;

    Ok(json!({ "jwt": jwt })
        .to_string()
        .customize()
        .append_header(("Content-Type", "application/json"))
        .with_status(StatusCode::OK))
}

async fn create_session(
    state: web::Data<AppData<'_>>,
    database: &mut PooledConnection<ConnectionManager<PgConnection>>,
    login_data: &LoginRequest,
    user: UserSelect,
) -> super::Result<String> {
    let dev_hash = state
        .xxh3_128bits(login_data.device_signature.into())
        .await
        .to_ne_bytes();

    let session_id = {
        use crate::schema::sessions::dsl::*;
        let record = SessionInsert::create(
            &user.id,
            &login_data.device_os,
            &login_data.device_name,
            &dev_hash,
        );

        diesel::insert_into(sessions)
            .values(&record)
            .on_conflict(device_hash)
            .do_update()
            .set(last_login.eq(Utc::now().naive_utc()))
            .returning(id)
            .get_result::<uuid::Uuid>(&mut *database)
            .unwrap()
    };

    state.jwt_encode(&JwtClaims::new(session_id))
}

#[actix_web::delete("/logout")]
async fn delete_logout(
    (state, user): (web::Data<AppData<'_>>, UserClaims),
) -> super::Result<impl Responder> {
    let mut connection = &mut *state.connect_database();
    use crate::schema::sessions;

    diesel::delete(sessions::table.filter(sessions::id.eq(user.session_id)))
        .execute(connection).ok();

    Ok("".customize().with_status(StatusCode::NO_CONTENT))
}

#[actix_web::get("/current")]
async fn get_current(
    (state, user): (web::Data<AppData<'_>>, UserClaims),
) -> super::Result<impl Responder> {
    use crate::schema::users;

    let database = &mut *state.connect_database();
    let (username, first_name, last_name, assigned_role) = match users::table
        .select((
            users::username,
            users::first_name,
            users::last_name,
            users::assigned_role,
        ))
        .filter(users::id.eq(user.user_id))
        .first::<(String, String, String, UserRole)>(database)
        .optional()
    {
        Ok(Some(val)) => val,
        Ok(None) => {
            return Err(ResponseError::new(
                "Unable to retrieve user data",
                "Unable to retrieve data",
                LogLevel::Error,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
        Err(_) => {
            return Err(ResponseError::new(
                "Unable to retrieve user data",
                "Unable to retrieve data",
                LogLevel::Error,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    Ok(json!({
        "id": user.user_id,
        "username": username,
        "first-name": first_name,
        "last-name": last_name,
        "assigned-role": assigned_role
    })
    .to_string()
    .customize()
    .append_header(("Content-Type", "application/json"))
    .with_status(StatusCode::OK))
}

#[actix_web::post("/register")]
async fn post_register(
    (state, request, user): (
        web::Data<AppData<'_>>,
        web::Json<CreateUserRequest>,
        UserClaims,
    ),
) -> super::Result<impl Responder> {
    use crate::schema::users::dsl::*;

    let mut connection = state.connect_database();
    let model = request.model(state);

    if user.assigned_role == UserRole::SecurityGuard {
        return Err(crate::logging::ResponseError::unauthorized(user));
    }

    if user.assigned_role == UserRole::SecurityGuard
        && request.assigned_role == UserRole::SystemAdmin
    {
        return Err(crate::logging::ResponseError::unauthorized(user));
    }

    let user_id = diesel::insert_into(users)
        .values(&model)
        .returning(id)
        .get_result::<uuid::Uuid>(&mut connection)
        .map_err(|err: diesel::result::Error| match err {
            diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                crate::logging::ResponseError::new(
                    "Username taken",
                    "Username taken",
                    LogLevel::Information,
                    StatusCode::CONFLICT,
                )
            }
            _ => crate::logging::ResponseError::new(
                "Failed to register user",
                "Failed to register user",
                LogLevel::Error,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        })?;

    Ok(web::Json(CreateUserOk { id: user_id })
        .customize()
        .append_header(("Content-Type", "application/json"))
        .with_status(StatusCode::OK))
}

#[actix_web::get("/guard")]
async fn get_guard(
    (state, user, query): (web::Data<AppData<'_>>, UserClaims, web::Query<GuardQuery>),
) -> super::Result<impl Responder> {
    use crate::schema::users::dsl::*;

    if user.assigned_role == UserRole::SecurityGuard {
        return Err(crate::logging::ResponseError::unauthorized(user));
    }

    let mut connection = state.connect_database();

    match query.filter {
        Some(Filter::Assigned) => {
            let guards = users
                .filter(deactivated.eq(false))
                .filter(
                    assigned_area
                        .is_not_null()
                        .and(assigned_role.eq(UserRole::SecurityGuard)),
                )
                .select((id, first_name, last_name, assigned_area))
                .load::<GuardSelect>(&mut connection)
                .unwrap();

            Ok(serde_json::to_string(&guards)
                .customize()
                .with_status(StatusCode::OK))
        }
        Some(Filter::Unassigned) => {
            let guards = users
                .filter(deactivated.eq(false))
                .filter(
                    assigned_area
                        .is_null()
                        .and(assigned_role.eq(UserRole::SecurityGuard)),
                )
                .select((id, first_name, last_name, assigned_area))
                .load::<GuardSelect>(&mut connection)
                .unwrap();

            Ok(serde_json::to_string(&guards)
                .customize()
                .with_status(StatusCode::OK))
        }
        None => {
            let guards = users
                .filter(deactivated.eq(false))
                .filter(assigned_role.eq(UserRole::SecurityGuard))
                .select((id, first_name, last_name, assigned_area))
                .load::<GuardSelect>(&mut connection)
                .unwrap();

            Ok(serde_json::to_string(&guards)
                .customize()
                .with_status(StatusCode::OK))
        }
    }
}

#[actix_web::get("/avatar")]
async fn get_avatar(
    (state, query, user): (
        web::Data<AppData<'_>>,
        web::Query<GetAvatarQuery>,
        UserClaims,
    ),
) -> super::Result<impl Responder> {
    use crate::schema::users;

    let user_id = query.id.unwrap_or(user.user_id);

    let mut connection = state.connect_database();

    let avatar: Option<Vec<u8>> = users::table
        .filter(users::id.eq(user_id))
        .select(users::avatar)
        .get_result(&mut connection)
        .or(Err(crate::logging::ResponseError::server_error()))?;

    if let Some(bytes) = avatar {
        return Ok(HttpResponse::build(StatusCode::OK)
            .content_type("image/jpeg")
            .body(bytes));
    }

    let first_name: String = users::table
        .filter(users::id.eq(user_id))
        .select(users::first_name)
        .get_result(&mut connection)
        .unwrap();

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("image/jpeg")
        .body(state.draw_default_avatar(&first_name)?))
}

#[actix_web::patch("/avatar")]
async fn patch_avatar(
    (state, mut payload, user): (
        web::Data<AppData<'_>>,
        actix_multipart::Multipart,
        UserClaims,
    ),
) -> super::Result<impl Responder> {
    while let Some(item) = payload.next().await {
        use crate::schema::users;

        let mut field = item.unwrap();

        if field.name() == "image" {
            let mut upload_bytes = Cursor::new(Vec::<u8>::new());

            while let Some(chunk) = field.next().await {
                let chunk = chunk.or(Err(crate::logging::ResponseError::invalid_field_format(
                    "image",
                )))?;

                upload_bytes.write_all(&chunk).ok();
            }

            let mut image_bytes = Vec::<u8>::new();

            {
                upload_bytes
                    .rewind()
                    .or(Err(crate::logging::ResponseError::server_error()))?;
                let mut dynamic_image = image::io::Reader::new(upload_bytes)
                    .with_guessed_format()
                    .or(Err(crate::logging::ResponseError::invalid_field_format(
                        "image",
                    )))?
                    .decode()
                    .or(Err(crate::logging::ResponseError::invalid_field_format(
                        "image",
                    )))?;

                let (w, h) = (dynamic_image.width(), dynamic_image.height());

                match w.cmp(&h) {
                    Ordering::Equal => {
                        if w > 256 {
                            dynamic_image = dynamic_image.thumbnail(256, 256);
                        }
                    }
                    Ordering::Less => {
                        dynamic_image = dynamic_image.crop(0, (h - w) / 2, w, w);

                        if w > 256 {
                            dynamic_image = dynamic_image.thumbnail(256, 256);
                        }
                    }
                    Ordering::Greater => {
                        dynamic_image = dynamic_image.crop((w - h) / 2, 0, h, h);

                        if h > 256 {
                            dynamic_image = dynamic_image.thumbnail(256, 256);
                        }
                    }
                }
                let mut writer = Cursor::new(Vec::<u8>::new());

                dynamic_image
                    .write_to(&mut writer, ImageOutputFormat::Jpeg(100))
                    .or(Err(crate::logging::ResponseError::server_error()))?;

                writer
                    .rewind()
                    .or(Err(crate::logging::ResponseError::server_error()))?;

                writer
                    .read_to_end(&mut image_bytes)
                    .or(Err(crate::logging::ResponseError::server_error()))?;
            }

            let mut connection = state.connect_database();

            return match diesel::update(users::table.filter(users::id.eq(user.user_id)))
                .set(users::avatar.eq(image_bytes))
                .execute(&mut connection)
            {
                Ok(row_count) => {
                    if row_count == 1 {
                        Ok(HttpResponse::NoContent())
                    } else {
                        Err(crate::logging::ResponseError::value_do_not_exist("User"))
                    }
                }
                Err(_) => Err(crate::logging::ResponseError::server_error()),
            };
        }
    }

    Err(crate::logging::ResponseError::new(
        "Bad request",
        "Bad request",
        LogLevel::Information,
        StatusCode::BAD_REQUEST,
    ))
}

#[actix_web::delete("/avatar")]
async fn delete_avatar(
    (state, user): (web::Data<AppData<'_>>, UserClaims),
) -> super::Result<impl Responder> {
    let mut connection = state.connect_database();

    use crate::schema::users;

    match diesel::update(users::table.filter(users::id.eq(user.user_id)))
        .set(users::avatar.eq(Option::<Vec<u8>>::None))
        .execute(&mut connection)
    {
        Ok(row_count) => {
            if row_count == 1 {
                Ok(HttpResponse::NoContent())
            } else {
                Err(crate::logging::ResponseError::value_do_not_exist("User"))
            }
        }
        Err(_) => Err(crate::logging::ResponseError::server_error()),
    }
}

pub fn scope() -> actix_web::Scope {
    web::scope("/users")
        .service(post_login)
        .service(get_current)
        .service(post_register)
        .service(get_guard)
        .service(get_avatar)
        .service(patch_avatar)
        .service(delete_avatar)
        .service(delete_logout)
}
