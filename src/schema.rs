// @generated automatically by Diesel CLI.

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        created_time -> Timestamp,
        last_login -> Timestamp,
        logout_time -> Nullable<Timestamp>,
        device_os -> Int2,
        device_name -> Varchar,
        device_sig -> Bytea,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        first_name -> Varchar,
        last_name -> Varchar,
        password_hash -> Bytea,
        assigned_role -> Int2,
    }
}

diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    sessions,
    users,
);
