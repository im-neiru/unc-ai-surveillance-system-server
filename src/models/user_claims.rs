use diesel::{Queryable, AsChangeset};

#[derive(Debug, Queryable, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
pub struct UserClaims {
    pub id: uuid::Uuid,
    pub assigned_role: super::UserRole,
}