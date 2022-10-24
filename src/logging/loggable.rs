
pub trait Loggable {
    fn message(&self) -> String;
    fn level(&self) -> super::LogLevel;
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;
}