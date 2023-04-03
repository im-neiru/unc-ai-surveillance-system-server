CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- CREATE USER unc_client WITH PASSWORD 'g1PxL1Lyvd8YqZ0U2x';
-- Create tables
CREATE TABLE areas(
    code VARCHAR(10) NOT NULL,
    name VARCHAR(128) NOT NULL,
    PRIMARY KEY(code)
);
CREATE TABLE users(
    id uuid DEFAULT uuid_generate_v4(),
    username VARCHAR(24) NOT NULL,
    first_name VARCHAR(80) NOT NULL,
    last_name VARCHAR(80) NOT NULL,
    password_hash BYTEA NOT NULL,
    deactivated BOOLEAN NOT NULL,
    assigned_role SMALLINT NOT NULL CHECK(assigned_role IN (1, 2, 3)),
    assigned_area VARCHAR(10) REFERENCES areas(code),
    PRIMARY KEY (id),
    UNIQUE(username)
);
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
CREATE TABLE violations(
    id uuid DEFAULT uuid_generate_v4(),
    area_code VARCHAR(10) NOT NULL REFERENCES areas(code),
    violation_kind SMALLINT NOT NULL CHECK(violation_kind IN (1, 2)),
    date_time TIMESTAMP NOT NULL,
    image_bytes BYTEA NOT NULL,
    identified BOOLEAN NOT NULL,
    personnel_id uuid REFERENCES users(id),
    first_name VARCHAR(48),
    last_name VARCHAR(48),
    category SMALLINT CHECK(category IN (1, 2, 3, 4)),
    PRIMARY KEY(id)
);
CREATE TABLE cameras(
    id integer NOT NULL,
    area_code VARCHAR(10) NOT NULL REFERENCES areas(code),
    camera_url VARCHAR(512) NOT NULL,
    deactivated BOOLEAN NOT NULL,
    PRIMARY KEY(id)
);
-- Sample users
DO $$
DECLARE password_argon2 BYTEA := decode(
        'kP/piFX/pcdVCl+eId23LQQX3GbYcgSsgWI0/eBNbJ8PMgq1p371HL0QIKlHKe3IDRWSKypIbIvk9wWwJHvsRg==',
        'base64'
    );
BEGIN
INSERT INTO users(
        username,
        first_name,
        last_name,
        password_hash,
        deactivated,
        assigned_role
    )
VALUES ('vladimir', 'Vlad', 'Tepes', password_argon2, FALSE,  1),
    ('rio', 'Rio', 'LeBlanc', password_argon2, FALSE, 2),
    ('admin', 'Lien', 'Walker', password_argon2, FALSE, 3);
END $$;
-- Sample areas
INSERT INTO areas(code, name)
VALUES ('GT2', 'Secondary gate');
INSERT INTO areas(code, name)
VALUES ('GT1', 'Primary gate');
-- Configure privileges
GRANT SELECT,
    INSERT,
    UPDATE ON users TO unc_client;
GRANT SELECT,
    INSERT,
    UPDATE ON sessions TO unc_client;
GRANT SELECT,
    INSERT,
    UPDATE ON areas TO unc_client;
GRANT SELECT,
    INSERT,
    UPDATE ON violations TO unc_client;
GRANT SELECT,
    INSERT,
    UPDATE ON cameras TO unc_client;