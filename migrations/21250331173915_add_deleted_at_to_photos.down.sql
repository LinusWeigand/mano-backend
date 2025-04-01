-- Add down migration script here
ALTER TABLE photos
DROP COLUMN deleted_at;
