-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE
    IF NOT EXISTS viewers (
        id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
        email VARCHAR(255) NOT NULL UNIQUE,
        hashed VARCHAR(255) NOT NULL,
        salt VARCHAR(255) NOT NULL,
        created_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW(),
        last_login TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW()
    );
