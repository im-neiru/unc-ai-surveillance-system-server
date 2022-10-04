use actix_web::{HttpServer, App, Responder, HttpResponse};
use tokio;

#[actix_web::get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok().body("Test")
}

fn main() -> std::io::Result<()> {
    tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(start_server())
}

async fn start_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(root)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}