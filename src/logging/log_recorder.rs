pub struct LogRecorder {
    pub(self) entries: Vec<LogEntry>
}

pub(super) struct LogEntry {
    pub(super) level: LogLevel,
    pub(super) message: String,
    pub(super) timestamp: chrono::DateTime<chrono::Utc>
}

pub(super) enum LogLevel {
    Error,
    Warning,
    Information,
    Debug,
    Trace,
}

pub struct LogWriter<'a, const LEVEL: LogLevel> {
    owner: &'a LogRecorder
}
