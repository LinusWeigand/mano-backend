-- Add down migration script here
ALTER TABLE viewers DROP COLUMN IF EXISTS verified;
