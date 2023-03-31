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

use crate::logging::{ ResponseError, LogLevel };

use super::{PasswordHash, UserRole};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct UserInsert {
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub password_hash: PasswordHash,
    pub assigned_role: UserRole,
    pub assigned_area: Option<String>,
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
    pub assigned_area: Option<String>,
}

impl UserSelect {
    pub fn select_by_username(connection: &mut PgConnection, username: &str) -> Result<Self, ResponseError> {
        use crate::schema::users::dsl;

        match dsl::users.filter(dsl::username.eq(username))
        .first::<Self>(connection).optional() {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(ResponseError::new(
                "A user entered incorrect username",
                "Invalid username or password",
                LogLevel::Error,
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
            Err(err) => Err(ResponseError::new(
                err.to_string().as_str(),
                "Failed to retrieved data",
                LogLevel::Error,
                StatusCode::INTERNAL_SERVER_ERROR
                )
            ),
        }
    }
}
