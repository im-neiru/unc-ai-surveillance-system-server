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

pub struct LogWriter<'a, const LEVEL: LogLevel> {
    recorder: &'a mut LogRecorder
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

    #[inline]
    pub(self) fn create_writer<'a, const LEVEL: LogLevel>(&'a mut self) -> LogWriter<'a, LEVEL>  {
        LogWriter { recorder: self }
    }
}