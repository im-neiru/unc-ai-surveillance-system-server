use std::future::Future;

use actix_web::web::Data;
use actix_web::FromRequest;
use tokio::sync::Mutex;

pub struct LogRecorder {
    entries: Vec<LogEntry>
}

pub(super) struct LogEntry {
    pub(super) level: LogLevel,
    pub(super) message: String,
    pub(super) timestamp: chrono::DateTime<chrono::Utc>
}

#[derive(Copy, Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warning,
    Information,
    Debug,
    Trace,
}

pub struct LogWriter<const LEVEL: LogLevel> {
    recorder: Data<Mutex<LogRecorder>>
}

impl LogRecorder {
    #[inline]
    pub(self) fn write(&mut self,
        level: LogLevel,
        message: &str,
        timestamp: chrono::DateTime<chrono::Utc>) {
        self.entries.push(LogEntry { level, message: message.to_string(),
            timestamp
        });
    }
}

impl<const LEVEL: LogLevel> LogWriter<LEVEL> {
    #[inline]
    pub(super) async fn write(&mut self,
        message: &str, 
        timestamp: chrono::DateTime<chrono::Utc>) {
        self.recorder
            .lock()
            .await
            .write(LEVEL, message, timestamp)
    }
}

impl<const LEVEL: LogLevel> FromRequest for LogWriter<LEVEL> {
    type Error = actix_web::Error;

    type Future = std::pin::Pin<Box<dyn Future<Output = Result<LogWriter<LEVEL>, actix_web::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            let writer = Self {
                recorder: req.app_data::<Data<Mutex<LogRecorder>>>()
                    .expect("Unable to retrieve the LogRecorder")
                    .clone()
            };

            Ok(writer)
        })
    }

    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        Self::from_request(req, &mut actix_web::dev::Payload::None)
    }
}