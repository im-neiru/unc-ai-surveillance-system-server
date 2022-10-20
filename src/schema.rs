// @generated automatically by Diesel CLI.

diesel::table! {
    area (code) {
        code -> Varchar,
        name -> Varchar,
    }
}

diesel::table! {
    protocol_violations (id) {
        id -> Uuid,
        personnel_id -> Uuid,
        date_time -> Timestamp,
        area_code -> Varchar,
        category -> Int2,
    }
}

diesel::table! {
    protocol_violators (id) {
        id -> Uuid,
        first_name -> Varchar,
        last_name -> Varchar,
        category -> Varchar,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        created_time -> Timestamp,
        last_login -> Timestamp,
        logout_time -> Nullable<Timestamp>,
        device_os -> Int2,
        device_name -> Varchar,
        device_hash -> Bytea,
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

diesel::joinable!(protocol_violations -> area (area_code));
diesel::joinable!(protocol_violations -> users (personnel_id));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    area,
    protocol_violations,
    protocol_violators,
    sessions,
    users,
);
