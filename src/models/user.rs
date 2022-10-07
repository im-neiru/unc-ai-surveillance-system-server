use diesel::Insertable;
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

#[derive(diesel::Queryable)]
#[diesel(table_name = crate::schema::users)]
pub struct UserSelect<'a> {
    pub id : uuid::Uuid,
    pub username: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub password_hash: PasswordHash,
    pub assigned_role: UserRole,
}