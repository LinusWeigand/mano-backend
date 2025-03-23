-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS profiles (
  id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4 ()),
  viewer_id UUID REFERENCES viewers (id) ON DELETE CASCADE,
  name VARCHAR(100) NOT NULL,
  rechtsform_id UUID REFERENCES rechtsformen (id) ON DELETE SET NULL,
  email VARCHAR(100) NOT NULL,
  telefon VARCHAR(100) NOT NULL UNIQUE,
  craft_id UUID REFERENCES crafts (id) ON DELETE SET NULL,
  experience SMALLINT NOT NULL CHECK (experience <= 1000),
  location VARCHAR(100) NOT NULL,
  website VARCHAR(100),
  instagram VARCHAR(100),
  bio VARCHAR(500) NOT NULL,
  handwerks_karten_nummer VARCHAR(100) NOT NULL UNIQUE,
  accepted BOOLEAN NOT NULL DEFAULT FALSE,
  version SMALLINT NOT NULL DEFAULT 1,
  created_at TIMESTAMP
  WITH
    TIME ZONE DEFAULT NOW (),
  updated_at TIMESTAMP
  WITH
    TIME ZONE DEFAULT NOW ()
);
