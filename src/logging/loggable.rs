
pub trait Loggable {
    fn message<'a>(&'a self) -> &'a str;
    fn level(&self) -> super::LogLevel;
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;
}