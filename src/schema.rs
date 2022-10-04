// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_role"))]
    pub struct UserRole;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserRole;

    users (id) {
        id -> Uuid,
        first_name -> Varchar,
        last_name -> Varchar,
        password_hash -> Bpchar,
        assigned_role -> UserRole,
    }
}
