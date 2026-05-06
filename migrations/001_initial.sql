-- Migration 001: Initial Schema
-- Creates the channels table and indexes

CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE IF NOT EXISTS channels (
    id              VARCHAR(8)      PRIMARY KEY,
    name            VARCHAR(100)    NOT NULL,
    password_hash   VARCHAR(255),
    link_limitation JSONB          DEFAULT '[]'::jsonb,
    channel_type    VARCHAR(50),
    location        VARCHAR(200),
    teacher         VARCHAR(100),
    creator_ip      VARCHAR(45)     NOT NULL,
    message_count   INTEGER         DEFAULT 0,
    last_activity   TIMESTAMPTZ     DEFAULT NOW(),
    created_at      TIMESTAMPTZ     DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_channels_type ON channels (channel_type) WHERE channel_type IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_channels_name_trgm ON channels USING gin (name gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_channels_last_activity ON channels (last_activity);

CREATE INDEX IF NOT EXISTS idx_channels_creator_ip ON channels (creator_ip);

COMMENT ON TABLE channels IS 'Stores channel metadata. Messages are kept in-memory.';
COMMENT ON COLUMN channels.password_hash IS 'bcrypt hash of channel password. NULL means no password.';
COMMENT ON COLUMN channels.link_limitation IS 'Array of allowed QR code link domains for security.';
COMMENT ON COLUMN channels.message_count IS 'Denormalized counter, synced with in-memory store.';
