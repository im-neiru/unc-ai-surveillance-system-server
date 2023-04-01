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
        assigned_area -> Nullable<Varchar>,
    }
}

diesel::table! {
    violations (id) {
        id -> Uuid,
        area_code -> Varchar,
        violation_kind -> Int2,
        date_time -> Timestamp,
        image_bytes -> Bytea,
        identified -> Bool,
        personnel_id -> Nullable<Uuid>,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        category -> Nullable<Varchar>,
    }
}

diesel::joinable!(cameras -> areas (area_code));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(users -> areas (assigned_area));
diesel::joinable!(violations -> areas (area_code));
diesel::joinable!(violations -> users (personnel_id));

diesel::allow_tables_to_appear_in_same_query!(
    areas,
    cameras,
    sessions,
    users,
    violations,
);
