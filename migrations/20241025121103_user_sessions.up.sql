-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS user_sessions (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
  viewer_id UUID REFERENCES viewers (id) ON DELETE CASCADE NOT NULL,
  hashed_session_token VARCHAR(100) NOT NULL,
  salt VARCHAR(100) NOT NULL,
  version INT NOT NULL DEFAULT 1,
  created_at TIMESTAMP
  WITH
    TIME ZONE NOT NULL DEFAULT NOW (),
    expires_at TIMESTAMP
  WITH
    TIME ZONE NOT NULL DEFAULT (NOW () + INTERVAL '48hours')
);
