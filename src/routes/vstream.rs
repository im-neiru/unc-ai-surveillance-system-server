use actix_web::{
    web,
    Responder,
    http::StatusCode
};

use crate::traits::WithSeperator;

use crate::{
    logging::LogResult,
    media::Surveillance
};

#[actix_web::get("/cameras")]
async fn get_cameras(surveillance: web::Data<Surveillance>) -> LogResult<impl Responder> {
    todo!()
}

pub fn scope() -> actix_web::Scope {
    web::scope("/vstream")
        .service(get_cameras)
}
