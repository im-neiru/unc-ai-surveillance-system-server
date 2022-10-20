CREATE TABLE area(
    code VARCHAR(10) NOT NULL,
    name VARCHAR(128) NOT NULL,

    PRIMARY KEY(code)
);

CREATE TABLE protocol_violators(
    id uuid DEFAULT uuid_generate_v4(),
    first_name VARCHAR(48) NOT NULL,
    last_name VARCHAR(48) NOT NULL,
    category VARCHAR(8) NOT NULL CHECK(category IN ('student', 'visitor', 'faculty', 'staff')),

    PRIMARY KEY(id)
);

CREATE TABLE protocol_violations(
    id uuid DEFAULT uuid_generate_v4(),
    personnel_id uuid NOT NULL REFERENCES users(id),
    date_time TIMESTAMP NOT NULL,
    area_code VARCHAR(10) NOT NULL REFERENCES area(code),
    category SMALLINT NOT NULL CHECK(category IN (1, 2)),

    PRIMARY KEY(id)
);

GRANT
    SELECT, INSERT, UPDATE 
    ON area TO unc_client;

GRANT
    SELECT, INSERT, UPDATE 
    ON protocol_violators TO unc_client;

GRANT
    SELECT, INSERT, UPDATE 
    ON protocol_violations TO unc_client;