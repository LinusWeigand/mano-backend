-- Add up migration script here
ALTER TABLE viewers ADD COLUMN IF NOT EXISTS verified BOOLEAN NOT NULL DEFAULT FALSE;