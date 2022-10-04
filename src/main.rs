use actix_web::{HttpServer, App, Responder, HttpResponse};
use tokio;

// local imports
mod server_config;

use server_config::ServerConfig;

#[actix_web::get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok().body("Test")
}

fn main() -> std::io::Result<()> {
    let server_config = ServerConfig::load();

    tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(start_server(&server_config))
}

async fn start_server(server_config: &ServerConfig) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(root)
    })
    .bind(server_config.web_server.clone())?
    .run()
    .await
}