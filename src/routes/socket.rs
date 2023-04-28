use crate::models::UserClaims;
use crate::AppData;
use actix_web::web;
use actix_web::{dev::Payload, FromRequest, HttpRequest, HttpResponse};
use actix_web_actors::ws;

async fn index(
    request: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let app_data = web::Data::<AppData>::from_request(&request, &mut Payload::None)
        .await
        .unwrap();
    let claims = web::Data::<UserClaims>::from_request(&request, &mut Payload::None)
        .await
        .unwrap();
    let actor = app_data.create_socket(claims.get_ref());

    ws::start(actor, &request, stream)
}

pub fn resource() -> actix_web::Resource {
    web::resource("/socket").to(index)
}
