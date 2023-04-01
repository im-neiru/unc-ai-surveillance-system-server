use actix_web::http::StatusCode;
use actix_web::Responder;
use actix_web::{post, web};

use chrono::Utc;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::result::DatabaseErrorKind;
use diesel::BoolExpressionMethods;
use diesel::{ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::data::AppData;
use crate::logging::{LogLevel, ResponseError};
use crate::models::{
    DeviceOs, DeviceSignature, JwtClaims, SessionInsert, UserBasicSelect, UserClaims, UserInsert,
    UserRole, UserSelect,
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
    fn model(&self, state: web::Data<AppData>) -> UserInsert {
        UserInsert {
            username: self.username.clone(),
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            password_hash: state.argon2(&self.password),
            assigned_role: self.assigned_role,
            assigned_area: None,
        }
    }
}

#[post("/login")]
async fn post_login(
    (body, state): (web::Json<LoginRequest>, web::Data<AppData>),
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
    state: web::Data<AppData>,
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

#[actix_web::get("/current")]
async fn get_current(
    (state, user): (web::Data<AppData>, UserClaims),
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
        .filter(users::id.eq(user.id))
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
        "id": user.id,
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
    (state, request, user): (web::Data<AppData>, web::Json<CreateUserRequest>, UserClaims),
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

#[actix_web::get("/unassigned")]
async fn get_unassigned(
    (state, user): (web::Data<AppData>, UserClaims),
) -> super::Result<impl Responder> {
    use crate::schema::users::dsl::*;

    if user.assigned_role == UserRole::SecurityGuard {
        return Err(crate::logging::ResponseError::unauthorized(user));
    }

    let mut connection = state.connect_database();

    let guards = users
        .filter(
            assigned_area
                .is_null()
                .and(assigned_role.eq(UserRole::SecurityGuard)),
        )
        .select((id, first_name, last_name))
        .load::<UserBasicSelect>(&mut connection)
        .unwrap();

    Ok(serde_json::to_string(&guards)
        .customize()
        .with_status(StatusCode::OK))
}

pub fn scope() -> actix_web::Scope {
    web::scope("/users")
        .service(post_login)
        .service(get_current)
        .service(post_register)
        .service(get_unassigned)
}
