use std::sync::Mutex;

use actix_web::{HttpServer, App, Responder, HttpResponse};
use tokio;

// local imports
mod server_config;
mod app_state;
mod routes;
mod schema;

use server_config::ServerConfig;
use app_state::AppState;

fn main() -> std::io::Result<()> {
    let server_config = ServerConfig::load();

    tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(start_server(&server_config))
}

async fn start_server(server_config: &ServerConfig) -> std::io::Result<()> {
    let database_url = server_config.database_url.clone();

    HttpServer::new(move || {
        let data = actix_web::web::Data::new(Mutex::new(AppState::create(database_url.as_str())));

        App::new()
            .app_data(data)
            .service(routes::users::scope())
    })
    .bind(server_config.web_server.clone())?
    .run()
    .await
}