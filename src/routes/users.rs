use actix_web::{post, web};
use actix_web::{HttpResponse, Responder};

use diesel::{QueryDsl, RunQueryDsl, OptionalExtension};
use diesel::ExpressionMethods;

use serde::{Serialize, Deserialize};

use crate::data::AppData;
use crate::models::{UserSelect, DeviceSignature, DeviceOs};
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

    if let Some(user) = users.filter(username.eq(&body.username)).first::<UserSelect>(&mut database).optional().unwrap()
    {
        if argon2 == user.password_hash {
            todo!("Generate web tokens");
        }
    }
    else {
        return HttpResponse::Unauthorized();
    }

    HttpResponse::Ok()
}

pub fn scope() -> actix_web::Scope {
    web::scope("/users")
        .service(post_login)
}