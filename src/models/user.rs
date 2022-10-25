use actix_web::http::StatusCode;
use diesel::{
    Insertable,
    Queryable,
    AsChangeset,
    PgConnection,
    QueryDsl,
    RunQueryDsl,
    ExpressionMethods, OptionalExtension
};

use crate::logging::{LoggableResponseError, LogLevel};

use super::{PasswordHash, UserRole};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct UserInsert<'a> {
    pub username: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub password_hash: PasswordHash,
    pub assigned_role: UserRole,
}

#[derive(Debug, Queryable, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
pub struct UserSelect {
    pub id : uuid::Uuid,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub password_hash: PasswordHash,
    pub assigned_role: UserRole,
}

impl UserSelect {
    pub fn select_by_username(connection: &mut PgConnection, username: &str) -> Result<Self, LoggableResponseError> {
        use crate::schema::users::dsl;

        match dsl::users.filter(dsl::username.eq(username))
        .first::<Self>(connection).optional() {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(LoggableResponseError::new(
                "A user enter incorrect username",
                "Invalid username or password",
                LogLevel::Error,
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
            Err(err) => Err(LoggableResponseError::new(
                err.to_string().as_str(),
                "Failed to retrieved data",
                LogLevel::Error,
                StatusCode::INTERNAL_SERVER_ERROR
                )
            ),
        }
    }
}