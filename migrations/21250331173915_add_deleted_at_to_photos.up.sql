-- Add up migration script here
ALTER TABLE photos 
ADD COLUMN deleted_at TIMESTAMP WITH TIME ZONE;

-- Create index for better performance on cleanup queries
CREATE INDEX idx_photos_deleted_at ON photos (deleted_at) 
WHERE deleted_at IS NOT NULL;
