use actix_web::{post, web};
use actix_web::{HttpResponse, Responder};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LoginData {
    pub username: String,
    pub password: String,
}

#[post("/login")]
async fn post_login(state: web::Data<crate::AppState>, inputs: web::Json<LoginData>) -> impl Responder {
    HttpResponse::Ok().body("Test")
}

pub fn scope() -> actix_web::Scope {
    web::scope("/users")
        .service(post_login)
}