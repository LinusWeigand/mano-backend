-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS pre_registered (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
  viewer_id UUID REFERENCES viewers (id) ON DELETE CASCADE NOT NULL,
  verification_code_hashed VARCHAR(64) NOT NULL,
  salt VARCHAR(100) NOT NULL,
  was_used BOOLEAN NOT NULL DEFAULT FALSE,
  version SMALLINT NOT NULL DEFAULT 1,
  created_at TIMESTAMP
  WITH
    TIME ZONE NOT NULL DEFAULT NOW (),
  expires_at TIMESTAMP
  WITH
    TIME ZONE NOT NULL DEFAULT (NOW () + INTERVAL '2 hours')
);
