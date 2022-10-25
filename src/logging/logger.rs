use std::future::{ready, Ready};
use tokio::sync::Mutex;

use actix_web::Error;
use actix_web::web::Data;

use actix_web::dev::{
    forward_ready,
    Service,
    ServiceRequest,
    ServiceResponse,
    Transform
};

pub struct Logger;


impl<S, R> Transform<S, ServiceRequest> for Logger
where
    S: Service<ServiceRequest, Response = ServiceResponse<R>, Error = Error>,
    S::Future: 'static,
    R: 'static
{
    type Response = ServiceResponse<R>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggerMiddleware { service }))
    }
}

pub struct LoggerMiddleware<S> {
    service: S,
}

impl<S, R> Service<ServiceRequest> for LoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<R>, Error = Error>,
    S::Future: 'static,
    R: 'static
{
    type Response = ServiceResponse<R>;
    type Error = Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let future = self.service.call(request);

        Box::pin(async move {
            let response = future.await?;

            if let Some(error) = response.response().error() {    
                if let Some(log) = error.as_error::<super::LoggableResponseError>() {
                    let mut recorder = response.request()
                        .app_data::<Data<Mutex<super::LogRecorder>>>()
                        .expect("No log recorder attached")
                        .lock()
                        .await;

                    recorder.record(log, Some(response.request().path()));
                }
            }
            
            Ok(response)
        })
    }
}