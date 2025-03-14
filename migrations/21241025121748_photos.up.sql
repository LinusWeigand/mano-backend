-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS photos (
  id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4 ()),
  profile_id UUID REFERENCES profiles (id) ON DELETE CASCADE NOT NULL,
  file_name VARCHAR(100) NOT NULL,
  content_type VARCHAR(100) NOT NULL,
  photo_data BYTEA NOT NULL,
  version SMALLINT NOT NULL DEFAULT 1,
  created_at TIMESTAMP
  WITH
    TIME ZONE DEFAULT NOW ()
);
