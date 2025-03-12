-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS profiles (
  id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4 ()),
  viewer_id UUID REFERENCES viewers (id) ON DELETE CASCADE,
  name VARCHAR(100) NOT NULL,
  craft_id UUID REFERENCES crafts (id) ON DELETE SET NULL,
  location VARCHAR(100) NOT NULL,
  website VARCHAR(100),
  instagram VARCHAR(100),
  bio TEXT NOT NULL,
  experience INT NOT NULL CHECK (experience <= 1000),
  google_ratings TEXT,
  created_at TIMESTAMP
  WITH
    TIME ZONE DEFAULT NOW (),
    updated_at TIMESTAMP
  WITH
    TIME ZONE DEFAULT NOW ()
);
