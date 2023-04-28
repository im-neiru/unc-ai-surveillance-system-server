pub(crate) mod areas;
pub(crate) mod logs;
pub(crate) mod users;
pub(crate) mod violations;
pub(crate) mod socket;

pub(crate) type Result<T, E = crate::logging::ResponseError> = std::result::Result<T, E>;
