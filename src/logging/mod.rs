mod log_recorder;
mod loggable_error;

pub use log_recorder::LogRecorder;
pub use log_recorder::LogWriter;
pub use log_recorder::LogLevel;

pub use loggable_error::Loggable;
pub use loggable_error::LoggableWithResponse;
pub use loggable_error::LogResponseError;

pub type LoggedResult<T> = Result<T, LogResponseError>;