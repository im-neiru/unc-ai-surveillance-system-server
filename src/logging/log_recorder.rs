use chrono::TimeZone;

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

    #[inline]
    pub fn retrieve(&self, index: i32, mut length: i32) -> String {
        if (index + length) as usize > self.entries.len() {
            length = self.entries.len() as i32- index;
        }

        let start = index as usize;
        let end = length as usize + start;

        let mut json = String::with_capacity(64);

        json.push_str("{logs:[");

        for entry in &self.entries[start..end] {
            let level = match entry.level {
                super::LogLevel::Error => "error",
                super::LogLevel::Warning => "warn",
                super::LogLevel::Information => "info",
                super::LogLevel::Debug => "debug",
                super::LogLevel::Trace => "trace",
            };

            let timestamp = entry
                .timestamp
                .to_rfc3339();

            let element = serde_json::json!({
                "level": level,
                "message": entry.message,
                "timestamp": timestamp,
                "path": entry.path,
            }).to_string();

            json.push_str(&element);
            json.push(',');
        }

        json.pop();
        json.push_str("]}");

        return json;
    }
}