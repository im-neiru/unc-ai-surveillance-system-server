use actix_web::{App, HttpServer};
use logging::LogRecorder;
use tokio::{self, sync::Mutex};

// local imports
mod data;
mod logging;
mod models;
mod notifier;
mod routes;
mod schema;
mod server_config;
mod traits;

use logging::LoggableError as Error;
type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests;

use data::AppData;
use server_config::ServerConfig;

use crate::notifier::ActiveEntry;

fn main() -> std::io::Result<()> {
    let server_config = ServerConfig::load();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(start_server(&server_config))
}

async fn start_server(server_config: &ServerConfig) -> std::io::Result<()> {
    let data = actix_web::web::Data::new(AppData::create(&server_config.database_url));
    let logger = actix_web::web::Data::new(Mutex::new(LogRecorder::new()));

    let data2 = data.clone();
    tokio::spawn(async move {
        println!("Press any key to send");
        loop {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();

            data2
                .notifier_mut()
                .await
                .notify(notifier::Notification::NewActivation(
                    [
                        ActiveEntry {
                            id: uuid::Uuid::new_v4(),
                            activity: false,
                        },
                        ActiveEntry {
                            id: uuid::Uuid::new_v4(),
                            activity: true,
                        },
                    ]
                    .to_vec(),
                ))
        }
    });

    /*let surveillance = actix_web::web::Data::new({
        let mut logger = logger.lock().await;

        media::Surveillance::new()
            .await
            .log_on_error(&mut logger)
            .expect("Failed to start surveillance")
    })*/

    // insert
    //insert_sample_violations(data.clone());

    println!("Server started at port {}", server_config.port);
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
            .service(routes::socket::resource())
    })
    .bind(server_config.actix_socket_addr())?
    .run()
    .await
}

#[allow(unused)]
fn insert_sample_violations(data: actix_web::web::Data<AppData>) {
    {
        let img = image::io::Reader::open("samples/incorrect1.png")
            .unwrap()
            .decode()
            .unwrap();

        data.store_violation(
            "GT2".to_owned(),
            models::ViolationKind::FacemaskProtocol,
            img.to_rgb8(),
        )
    }

    {
        let img = image::io::Reader::open("samples/nofacemask.png")
            .unwrap()
            .decode()
            .unwrap();

        data.store_violation(
            "GT1".to_owned(),
            models::ViolationKind::FacemaskProtocol,
            img.to_rgb8(),
        )
    }
}
