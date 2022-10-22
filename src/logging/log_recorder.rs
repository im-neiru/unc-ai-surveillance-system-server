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