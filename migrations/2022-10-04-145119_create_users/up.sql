CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE user_role AS ENUM('security-guard', 'security-head');

CREATE TABLE users(
    id uuid DEFAULT uuid_generate_v4(),
    username VARCHAR(24) NOT NULL,
    first_name VARCHAR(80) NOT NULL,
    last_name VARCHAR(80) NOT NULL,
    password_hash BYTEA NOT NULL,
    assigned_role user_role NOT NULL,

    PRIMARY KEY (id)
);