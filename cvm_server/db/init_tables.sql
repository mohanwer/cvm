CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS applications
(
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS clients
(
    id                 UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    app_id             UUID REFERENCES applications (id) NOT NULL,
    created_at         TIMESTAMP with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at         TIMESTAMP with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    build_version      Varchar(255) NOT NULL,
    version            VARCHAR(255) NOT NULL,
    enabled            BOOLEAN DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS application_versions
(
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    app_id        UUID REFERENCES applications (id) NOT NULL,
    version       VARCHAR(255) NOT NULL,
    latest        BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS application_builds
(
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    app_version_id  UUID REFERENCES application_versions (id) NOT NULL,
    build_version varchar(255) NOT NULL,
    success_count INTEGER          DEFAULT 0,
    failed_count  INTEGER          DEFAULT 0,
    url           VARCHAR(255) NOT NULL,
    disabled      BOOLEAN          DEFAULT FALSE
);