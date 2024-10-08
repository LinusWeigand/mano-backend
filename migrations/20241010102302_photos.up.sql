-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE
    IF NOT EXISTS photos (
        id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
        profile_id UUID REFERENCES profiles(id) ON DELETE CASCADE NOT NULL,
        file_name TEXT NOT NULL UNIQUE,
        content_type TEXT NOT NULL,
        photo_data BYTEA NOT NULL,
        created_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW()
    );

