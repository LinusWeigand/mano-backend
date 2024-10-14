-- Add up migration script here
ALTER TABLE viewers ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE viewers
SET is_admin = TRUE
WHERE email IN ('linus@couchtec.com', 'matteo.levi.golisano@gmail.com');
