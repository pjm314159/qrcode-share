-- Migration 002: Channel Cleanup Function
-- Creates a function to clean up inactive channels

CREATE OR REPLACE FUNCTION cleanup_inactive_channels()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM channels
    WHERE last_activity < NOW() - INTERVAL '30 days';

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_inactive_channels() IS 'Deletes channels inactive for more than 30 days. Returns count of deleted channels.';
