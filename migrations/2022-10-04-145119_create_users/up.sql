CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users(
    id uuid DEFAULT uuid_generate_v4(),
    username VARCHAR(24) NOT NULL,
    first_name VARCHAR(80) NOT NULL,
    last_name VARCHAR(80) NOT NULL,
    password_hash BYTEA NOT NULL,
    assigned_role SMALLINT NOT NULL CHECK(assigned_role IN (1, 2, 3)),

    PRIMARY KEY (id),
    UNIQUE(username)
);


-- Sample users

INSERT INTO users(username, first_name, last_name, password_hash, assigned_role)
VALUES ('vladimir', 'Vlad', 'Tepes', 
decode('Wo+1JdceNVhCjdKSAPq6bDUJJKgFfwhJyCPjCXVQ6tp7lAeHu6YMsmJB3AYusuLsO9ym6EemeN2FJnWOx/Ta2g==', 'base64'), 1);