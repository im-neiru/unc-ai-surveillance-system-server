use actix_web::{
    web,
    Responder,
    http::StatusCode
};

use crate::traits::WithSeperator;

use crate::media::Surveillance;

#[actix_web::get("/cameras")]
async fn get_cameras(surveillance: web::Data<Surveillance>) -> super::Result<impl Responder> {
    Ok(format!("cameras: [{}]", surveillance.iter()
    .map(|(id, _camera)| {
        //TODO: retrieve camera name
        format!("id: {}", id)
    })
    .with_seperator(|| String::from(", "))
    .fold(String::default(), | acc, item | acc + &item))
    .customize()
    .append_header(("Content-Type", "application/json"))
    .with_status(StatusCode::OK))
}

pub fn scope() -> actix_web::Scope {
    web::scope("/vstream")
        .service(get_cameras)
}
