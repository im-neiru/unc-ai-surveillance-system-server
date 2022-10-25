pub struct LogRecorder {
    entries: Vec<LogEntry>
}

pub(super) struct LogEntry {
    pub(super) level: super::LogLevel,
    pub(super) message: String,
    pub(super) timestamp: chrono::DateTime<chrono::Utc>,
    pub(super) path: Option<String>
}

impl LogRecorder {

    #[inline]
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    #[inline]
    pub fn record<L>(&mut self, log: &L, path: Option<&str>) where L: super::Loggable + Sized {
        self.entries.push(LogEntry {
            level: log.level(),
            message: log.message().to_owned(),
            timestamp: log.timestamp(),
            path: match path {
                Some(val) => Some(val.to_string()),
                None => None
            }
        })
    }
}