use actix_web::{post, web};
use actix_web::{HttpResponse, Responder};

use chrono::Utc;
use diesel::r2d2::{PooledConnection, ConnectionManager};
use diesel::{QueryDsl, RunQueryDsl, OptionalExtension, PgConnection};
use diesel::ExpressionMethods;

use serde::{Serialize, Deserialize};

use crate::data::AppData;
use crate::models::{UserSelect, DeviceSignature, DeviceOs, JwtClaims, SessionInsert, UserClaims};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LoginData {
    pub username: String,
    pub password: String,
    pub device_os: DeviceOs,
    pub device_name: String,
    pub device_signature: DeviceSignature,
}

#[post("/login")]
async fn post_login((body, state): 
(web::Json<LoginData>, web::Data<AppData>)) -> impl Responder {
    use crate::schema::users::dsl::*;
    
    let argon2 = state.argon2(&body.password);
    let mut database = state.connect_database();
    let jwt;

    if let Some(user) = users.filter(username.eq(&body.username)).first::<UserSelect>(&mut database).optional().unwrap()
    {
        if argon2 != user.password_hash {
            return HttpResponse::Unauthorized().body("");
        }

        jwt = create_session(state, &mut database, &body, user).await;
    }
    else {
        return HttpResponse::Unauthorized().body("");
    }

    HttpResponse::Ok().body(jwt)
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
async fn get_info((state, user): (web::Data<AppData>, UserClaims)) -> impl Responder {
    use crate::schema::users;

    let database = &mut *state.connect_database();

    let (username, last_name, first_name) = users::table.select((
        users::username,
        users::first_name,
        users::last_name,
    )).filter(users::id.eq(user.id))
    .first::<(String, String, String)>(database)
    .unwrap();

    HttpResponse::Ok().json(serde_json::json!({
        "username": username,
        "firstname": first_name, 
        "last_name": last_name,
    }))
}

pub fn scope() -> actix_web::Scope {
    web::scope("/users")
        .service(post_login)
        .service(get_info)
}