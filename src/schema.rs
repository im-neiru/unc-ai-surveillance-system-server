// @generated automatically by Diesel CLI.

diesel::table! {
    areas (code) {
        code -> Varchar,
        name -> Varchar,
    }
}

diesel::table! {
    cameras (id) {
        id -> Int4,
        area_code -> Varchar,
        camera_url -> Varchar,
    }
}

diesel::table! {
    protocol_violations (id) {
        id -> Uuid,
        personnel_id -> Uuid,
        area_code -> Varchar,
        category -> Int2,
        date_time -> Timestamp,
        image_bytes -> Bytea,
    }
}

diesel::table! {
    protocol_violators (id) {
        id -> Uuid,
        violation -> Uuid,
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

diesel::joinable!(cameras -> areas (area_code));
diesel::joinable!(protocol_violations -> areas (area_code));
diesel::joinable!(protocol_violations -> users (personnel_id));
diesel::joinable!(protocol_violators -> protocol_violations (violation));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    areas,
    cameras,
    protocol_violations,
    protocol_violators,
    sessions,
    users,
);
