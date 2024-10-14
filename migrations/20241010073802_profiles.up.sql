-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE
    IF NOT EXISTS profiles (
        id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
        viewer_id UUID REFERENCES viewers(id) ON DELETE CASCADE NOT NULL,
        name VARCHAR(100) NOT NULL,
        craft VARCHAR(100) NOT NULL,
        location VARCHAR(100) NOT NULL,
        website VARCHAR(100),
        instagram VARCHAR(100),
        skills VARCHAR(20)[] NOT NULL,
        bio TEXT NOT NULL,
        experience SMALLINT NOT NULL CHECK (experience <= 100),
        created_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW()
    );