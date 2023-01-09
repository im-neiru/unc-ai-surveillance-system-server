pub(crate) mod users;
pub(crate) mod logs;
pub(crate) mod vstream;

type Result<T, E = crate::logging::ResponseError> = std::result::Result<T, E>;
