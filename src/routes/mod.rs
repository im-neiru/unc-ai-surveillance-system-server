pub(crate) mod users;
pub(crate) mod logs;
pub(crate) mod vstream;

type Result<T, E: actix_web::ResponseError = crate::logging::ResponseError> = std::result::Result<T, E>;
