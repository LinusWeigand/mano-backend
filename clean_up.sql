CREATE OR REPLACE FUNCTION cleanup_expired_photos()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM photos WHERE deleted_at <= NOW();
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_cleanup_photos
AFTER INSERT OR UPDATE ON photos
FOR EACH STATEMENT
EXECUTE FUNCTION cleanup_expired_photos();
