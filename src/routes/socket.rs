use crate::AppData;
use actix_web::web;
use actix_web::{dev::Payload, FromRequest, HttpRequest, HttpResponse};

async fn index(
    request: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let app_data = web::Data::<AppData>::from_request(&request, &mut Payload::None)
        .await
        .unwrap();

    let mut notifier = app_data.notifier_mut().await;
    notifier.add_socket(&request, stream).await
}

pub fn resource() -> actix_web::Resource {
    web::resource("/socket").to(index)
}
