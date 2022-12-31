use actix_web::http::StatusCode;
use actix_web::{post, web};
use actix_web::Responder;

use chrono::Utc;
use diesel::r2d2::{PooledConnection, ConnectionManager};
use diesel::{QueryDsl, RunQueryDsl, PgConnection, ExpressionMethods, OptionalExtension};

use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::data::AppData;
use crate::logging::{LogResult, LoggableResponseError, LogLevel};
use crate::models::{UserSelect, DeviceSignature, DeviceOs, JwtClaims, SessionInsert, UserClaims};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LoginData {
    pub username: String,
    pub password: String,
    #[serde(alias = "device-os")]
    pub device_os: DeviceOs,
    #[serde(alias = "device-name")]
    pub device_name: String,
    #[serde(alias = "device-signature")]
    pub device_signature: DeviceSignature,
}

#[post("/login")]
async fn post_login((body, state): (web::Json<LoginData>, web::Data<AppData>)) -> LogResult<impl Responder> {
    let mut database = state.connect_database();
    let user = UserSelect::select_by_username(&mut database, &body.username)?;

    state.validate_password(user.password_hash, &body.password).await?;
    let jwt = create_session(state, &mut database, &body, user).await?;

    Ok(json!({
        "jwt": jwt
    })
    .to_string()
    .customize()
    .append_header(("Content-Type", "application/json"))
    .with_status(StatusCode::OK))
}

async fn create_session(state: web::Data<AppData>,
    database: &mut PooledConnection<ConnectionManager<PgConnection>>,
    login_data: &LoginData,
    user: UserSelect) -> LogResult<String> {


    let dev_hash = state.xxh3_128bits(login_data.device_signature.into()).await.to_ne_bytes();

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

    Ok(state.jwt_encode(&JwtClaims::new(session_id) )?)
}

#[actix_web::get("/info")]
async fn get_info((state, user): (web::Data<AppData>, UserClaims)) -> LogResult<impl Responder> {
    use crate::schema::users;

    let database = &mut *state.connect_database();
    let (username, last_name, first_name) = match users::table.select((
        users::username,
        users::first_name,
        users::last_name,
    )).filter(users::id.eq(user.id))
    .first::<(String, String, String)>(database)
    .optional() {
        Ok(Some(val)) => val,
        Ok(None) => return Err(LoggableResponseError::new(
            "Unable to retrieve user data",
            "Unable to retrieve data",
            LogLevel::Error,
            StatusCode::INTERNAL_SERVER_ERROR)),
        Err(_) => return Err(LoggableResponseError::new(
            "Unable to retrieve user data",
            "Unable to retrieve data",
            LogLevel::Error,
            StatusCode::INTERNAL_SERVER_ERROR))
    };

    Ok(json!({
        "id": user.id,
        "username": username,
        "first-name": first_name,
        "last-name": last_name,
    })
    .to_string()
    .customize()
    .append_header(("Content-Type", "application/json"))
    .with_status(StatusCode::OK))
}

pub fn scope() -> actix_web::Scope {
    web::scope("/users")
        .service(post_login)
        .service(get_info)
}
