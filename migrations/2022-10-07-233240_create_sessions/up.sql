CREATE TABLE sessions(
    id uuid DEFAULT uuid_generate_v4(),
    user_id uuid NOT NULL REFERENCES users(id),
    created_time TIMESTAMP NOT NULL,
    last_login TIMESTAMP NOT NULL,
    logout_time TIMESTAMP,
    device_os SMALLINT NOT NULL CHECK(device_os IN (1, 2, 3)),
    device_name VARCHAR(64) NOT NULL,
    device_hash BYTEA NOT NULL,

    PRIMARY KEY(id),
    UNIQUE(device_hash)
);

GRANT
    SELECT, INSERT, UPDATE 
    ON sessions TO unc_client;