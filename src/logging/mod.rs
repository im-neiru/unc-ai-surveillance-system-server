mod log_recorder;
mod loggable;
mod logger;
mod log_level;
mod loggable_response_error;
mod server_error;

pub use logger::Logger as LogMiddleware;
pub use log_level::LogLevel;
pub use loggable::Loggable;
pub use log_recorder::LogRecorder;
pub use loggable_response_error::LoggableResponseError;
pub use server_error::ServerError;

pub type LogResult<R, E: Loggable = LoggableResponseError> = Result<R, E>;