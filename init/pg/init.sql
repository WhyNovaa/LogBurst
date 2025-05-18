CREATE TABLE IF NOT EXISTS roles
(
    id SERIAL PRIMARY KEY,
    name VARCHAR(20) NOT NULL UNIQUE
);

INSERT INTO roles (name)
VALUES ('Admin'),
    ('User')
ON CONFLICT DO NOTHING;


CREATE TABLE IF NOT EXISTS users
(
    id SERIAL PRIMARY KEY,
    login VARCHAR(60) NOT NULL UNIQUE,
    hashed_password VARCHAR(256) NOT NULL,
    role_id INT REFERENCES Roles(id)
);

INSERT INTO users (login, hashed_password, role_id)
VALUES ('admin', '8c6976e5b5410415bde908bd4dee15dfb167a9c873fc4bb8a81f6f2ab448a918', 1);