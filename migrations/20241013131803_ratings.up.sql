-- Add up migration script here
ALTER TABLE profiles ADD COLUMN IF NOT EXISTS google_ratings TEXT;