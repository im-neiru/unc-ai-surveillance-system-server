mod log_recorder;
mod loggable_error;
mod logger;

pub use logger::Logger;

pub use log_recorder::LogRecorder;
pub use log_recorder::LogWriter;
pub use log_recorder::LogLevel;

pub use loggable_error::Loggable;
pub use loggable_error::LoggableWithResponse;
pub use loggable_error::LogResponseError;

pub type LoggedResult<T> = Result<T, LogResponseError>;

#[macro_export]
macro_rules! try_log {
    ($res:expr, $logger:expr) => {
        match ($res) {
            Ok(obj) => obj,
            Err(err) => return Ok(err.log($logger).await?),
        }
    };
}