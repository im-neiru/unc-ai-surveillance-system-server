use diesel::{Insertable, Queryable, AsChangeset};
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