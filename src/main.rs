use actix_web::{HttpServer, App};
use tokio;

// local imports
mod server_config;
mod data;
mod routes;
mod models;
mod schema;
mod traits;

use server_config::ServerConfig;
use data::AppData;

fn main() -> std::io::Result<()> {
    let server_config = ServerConfig::load();

    tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(start_server(&server_config))
}

async fn start_server(server_config: &ServerConfig) -> std::io::Result<()> {
    let data = actix_web::web::Data::new(AppData::create(&server_config.database_url));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(routes::users::scope())
    })
    .bind(server_config.web_server.clone())?
    .run()
    .await
}