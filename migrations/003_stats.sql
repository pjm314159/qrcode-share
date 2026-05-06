-- Migration 003: Channel Stats View
-- Creates a materialized view for channel statistics

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_matviews WHERE matviewname = 'channel_stats') THEN
        CREATE MATERIALIZED VIEW channel_stats AS
        SELECT
            id,
            name,
            channel_type,
            password_hash IS NOT NULL AS has_password,
            message_count,
            last_activity,
            created_at
        FROM channels
        WITH DATA;
    END IF;
END $$;

CREATE UNIQUE INDEX IF NOT EXISTS idx_channel_stats_id ON channel_stats (id);

CREATE INDEX IF NOT EXISTS idx_channel_stats_type ON channel_stats (channel_type)
    WHERE channel_type IS NOT NULL;

COMMENT ON MATERIALIZED VIEW channel_stats IS 'Pre-computed channel statistics for fast reads. Refresh periodically or on demand.';
