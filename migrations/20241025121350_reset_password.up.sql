-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS reset_password (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
  viewer_id UUID NOT NULL REFERENCES viewers (id) ON DELETE CASCADE,
  hashed_reset_password_token VARCHAR(100) NOT NULL,
  salt VARCHAR(100) NOT NULL,
  was_used BOOLEAN NOT NULL DEFAULT FALSE,
  version SMALLINT NOT NULL DEFAULT 1,
  created_at TIMESTAMP
  WITH
    TIME ZONE NOT NULL DEFAULT NOW (),
    expires_at TIMESTAMP
  WITH
    TIME ZONE NOT NULL DEFAULT NOW () + INTERVAL '2hours'
);
