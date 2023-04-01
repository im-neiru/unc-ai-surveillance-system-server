use actix_web::http::StatusCode;
use diesel::{
    deserialize::FromSqlRow, sql_types::Text, AsChangeset, ExpressionMethods, Insertable,
    OptionalExtension, PgConnection, QueryDsl, Queryable, RunQueryDsl,
};
use serde::Serialize;

use crate::logging::{LogLevel, ResponseError};

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
    pub id: uuid::Uuid,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub password_hash: PasswordHash,
    pub assigned_role: UserRole,
    pub assigned_area: Option<String>,
}

impl UserSelect {
    pub fn select_by_username(
        connection: &mut PgConnection,
        username: &str,
    ) -> Result<Self, ResponseError> {
        use crate::schema::users::dsl;

        match dsl::users
            .filter(dsl::username.eq(username))
            .first::<Self>(connection)
            .optional()
        {
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
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserBasicSelect {
    pub id: uuid::Uuid,
    #[serde(alias = "last-name")]
    pub last_name: String,
    #[serde(alias = "first-name")]
    pub first_name: String,
}

impl FromSqlRow<(diesel::sql_types::Uuid, Text, Text), diesel::pg::Pg> for UserBasicSelect {
    fn build_from_row<'a>(
        row: &impl diesel::row::Row<'a, diesel::pg::Pg>,
    ) -> diesel::deserialize::Result<Self> {
        Ok(Self{
            id: row.get_value("id")?,
            last_name: row.get_value("last_name")?,
            first_name: row.get_value("first_name")?,
        })
    }
}
