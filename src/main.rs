use actix_web::{HttpServer, App};
use logging::LogRecorder;
use tokio::{self, sync::Mutex};

// local imports
mod server_config;
mod data;
mod routes;
mod models;
mod schema;
mod traits;
mod logging;
//mod media;

use logging::LoggableError as Error;
type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests;

use server_config::ServerConfig;
use data::AppData;

fn main() -> std::io::Result<()> {
    let server_config = ServerConfig::load();
    println!("Starting");
    
    tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(start_server(&server_config))
}

async fn start_server(server_config: &ServerConfig) -> std::io::Result<()> {
    let data = actix_web::web::Data::new(AppData::create(&server_config.database_url));
    let logger = actix_web::web::Data::new(Mutex::new(LogRecorder::new()));
    /*let surveillance = actix_web::web::Data::new({
        let mut logger = logger.lock().await;
        
        media::Surveillance::new()
            .await
            .log_on_error(&mut logger)
            .expect("Failed to start surveillance")
    })*/

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .app_data(logger.clone())
            //.app_data(surveillance.clone())
            .wrap(logging::LogMiddleware)
            .service(routes::users::scope())
            .service(routes::logs::scope())
            .service(routes::areas::scope())
            .service(routes::violations::scope())
    })
    .bind(server_config.actix_socket_addr())?
    .run()
    .await
}
