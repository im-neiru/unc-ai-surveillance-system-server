pub struct LogRecorder {
    entries: Vec<LogEntry>
}

pub(super) struct LogEntry {
    pub(super) level: super::LogLevel,
    pub(super) message: String,
    pub(super) timestamp: chrono::DateTime<chrono::Utc>
}

impl LogRecorder {

    #[inline]
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    #[inline]
    pub fn record<L>(&mut self, log: &L) where L: super::Loggable + Sized {
        self.entries.push(LogEntry {
            level: log.level(),
            message: log.message(),
            timestamp: log.timestamp(),
        })
    }
}