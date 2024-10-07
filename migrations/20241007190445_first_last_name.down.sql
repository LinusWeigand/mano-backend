-- Add down migration script here
ALTER TABLE viewers DROP COLUMN IF EXISTS first_name;
ALTER TABLE viewers DROP COLUMN IF EXISTS last_name;