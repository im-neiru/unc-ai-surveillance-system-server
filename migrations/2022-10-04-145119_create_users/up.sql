CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE user_role AS ENUM('security-guard', 'security-head');

CREATE TABLE users(
    id uuid DEFAULT uuid_generate_v4(),
    first_name VARCHAR(80) NOT NULL,
    last_name VARCHAR(80) NOT NULL,
    password_hash BYTEA NOT NULL,
    assigned_role user_role NOT NULL,

    PRIMARY KEY (id)
);