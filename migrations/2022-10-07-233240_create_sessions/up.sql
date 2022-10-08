CREATE TABLE sessions(
    id uuid DEFAULT uuid_generate_v4(),
    user_id uuid NOT NULL REFERENCES users(id),
    created_time TIMESTAMP NOT NULL,
    last_login TIMESTAMP NOT NULL,
    logout_time TIMESTAMP,
    device_type SMALLINT NOT NULL CHECK(device_type IN (1, 2)),
    device_name VARCHAR(64) NOT NULL,
    device_sig BYTEA NOT NULL,

    PRIMARY KEY(id)
);