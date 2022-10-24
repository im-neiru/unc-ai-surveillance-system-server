use actix_web::http::StatusCode;
use actix_web::{post, web};
use actix_web::Responder;

use chrono::Utc;
use diesel::r2d2::{PooledConnection, ConnectionManager};
use diesel::{QueryDsl, RunQueryDsl, PgConnection};
use diesel::ExpressionMethods;

use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::data::AppData;
use crate::logging::{LogWriter, LogLevel, LoggedResult, LoggableWithResponse};
use crate::models::{UserSelect, DeviceSignature, DeviceOs, JwtClaims, SessionInsert, UserClaims};
use crate::try_log;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LoginData {
    pub username: String,
    pub password: String,
    pub device_os: DeviceOs,
    pub device_name: String,
    pub device_signature: DeviceSignature,
}

#[post("/login")]
async fn post_login((body, state, mut log_info): 
(web::Json<LoginData>, web::Data<AppData>, LogWriter<{ LogLevel::Information }>)) -> LoggedResult<impl Responder> {
    let mut database = state.connect_database();
    let jwt;
    let user = try_log!(UserSelect::select_by_username(&mut database, &body.username), &mut log_info);
    
    try_log!(state.validate_password(user.password_hash, &body.password).await, &mut log_info);

    jwt = create_session(state, &mut database, &body, user).await;

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
    user: UserSelect) -> String {


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

    state.jwt_encode(&JwtClaims::new(session_id) )
}

#[actix_web::get("/info")]
async fn get_info((state, user, mut log_info): (web::Data<AppData>, UserClaims, LogWriter<{ LogLevel::Information}>)) -> LoggedResult<impl Responder> {
    use crate::schema::users;

    let database = &mut *state.connect_database();
    let (username, last_name, first_name) = match users::table.select((
        users::username,
        users::first_name,
        users::last_name,
    )).filter(users::id.eq(user.id))
    .first::<(String, String, String)>(database) {
        Ok(val) => val,
        Err(_) => return LoggableWithResponse::new(
            "Unable to retrieve user data",
            "Unable to retrieve data",
            StatusCode::INTERNAL_SERVER_ERROR)
            .log(&mut log_info).await,
    };
    
    Ok(json!({
        "username": username,
        "firstname": first_name, 
        "last_name": last_name,
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