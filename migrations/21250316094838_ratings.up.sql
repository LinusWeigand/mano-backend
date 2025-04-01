-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS ratings (
  id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4 (),
  rating FLOAT NOT NULL,
  review_count SMALLINT NOT NULL,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
  version SMALLINT NOT NULL DEFAULT 1,
  created_at TIMESTAMP 
  WITH
    TIME ZONE NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMP 
  WITH
    TIME ZONE NOT NULL DEFAULT NOW ()
);
