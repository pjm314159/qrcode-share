# Database Documentation

## Overview

Qrcode Share uses a **hybrid storage architecture**:

- **PostgreSQL**: Persistent storage for channel metadata and configuration
- **In-Memory (DashMap)**: Short-lived messages with 1-hour TTL, high-performance access

This design ensures low-latency message delivery while keeping channel data durable.

### Storage Distribution

| Data                  | Storage               | TTL                   | Reason                            |
| --------------------- | --------------------- | --------------------- | --------------------------------- |
| Channel metadata      | PostgreSQL            | 30 days (inactive)    | Needs persistence across restarts |
| Channel password      | PostgreSQL            | 30 days (inactive)    | Security, must survive restart    |
| Link limitation rules | PostgreSQL            | 30 days (inactive)    | Security, must survive restart    |
| Messages              | In-Memory (DashMap)   | 1 hour                | Short-lived, high throughput      |
| WebSocket subscribers | In-Memory (broadcast) | Connection lifetime   | Ephemeral, real-time only         |
| Rate limiter state    | In-Memory (DashMap)   | 1 hour sliding window | Ephemeral, per-request            |
| Metrics               | In-Memory (AtomicU64) | Process lifetime      | Ephemeral, reset on restart       |

***

## PostgreSQL Schema

### 1. channels

Stores channel metadata. This is the only persistent table.

```sql
CREATE TABLE channels (
    id              VARCHAR(8)      PRIMARY KEY,
    name            VARCHAR(100)    NOT NULL,
    password_hash   VARCHAR(255),               -- bcrypt hash, NULL = no password
    link_limitation JSONB          DEFAULT '[]', -- Array of allowed domains
    channel_type    VARCHAR(50),                -- Optional type tag
    location        VARCHAR(200),               -- Optional location
    teacher         VARCHAR(100),               -- Optional teacher name
    creator_ip      VARCHAR(45),                -- For rate limiting (no login)
    message_count   INTEGER        DEFAULT 0,   -- Denormalized counter
    last_activity   TIMESTAMPTZ    DEFAULT NOW(),
    created_at      TIMESTAMPTZ    DEFAULT NOW()
);
```

#### Column Details

| Column           | Type         | Nullable | Default | Description                                     |
| ---------------- | ------------ | -------- | ------- | ----------------------------------------------- |
| id               | VARCHAR(8)   | No       | -       | Unique channel ID, 8-char alphanumeric          |
| name             | VARCHAR(100) | No       | -       | Channel display name                            |
| password\_hash   | VARCHAR(255) | Yes      | NULL    | bcrypt hash of password. NULL means no password |
| link\_limitation | JSONB        | Yes      | `[]`    | Array of allowed link domains for security      |
| channel\_type    | VARCHAR(50)  | Yes      | NULL    | Type tag (e.g., "sign-in", "payment")           |
| location         | VARCHAR(200) | Yes      | NULL    | Physical location description                   |
| teacher          | VARCHAR(100) | Yes      | NULL    | Teacher or organizer name                       |
| creator\_ip      | VARCHAR(45)  | No       | -       | IP address of creator, for rate limiting        |
| message\_count   | INTEGER      | No       | 0       | Denormalized message count for fast reads       |
| last\_activity   | TIMESTAMPTZ  | No       | NOW()   | Last message or subscriber activity             |
| created\_at      | TIMESTAMPTZ  | No       | NOW()   | Channel creation time                           |

#### Indexes

```sql
-- Primary lookup by channel ID (already from PRIMARY KEY)
-- This is the hot path: every WebSocket connection starts here

-- List channels by type (for filtering)
CREATE INDEX idx_channels_type ON channels (channel_type) WHERE channel_type IS NOT NULL;

-- Search channels by name (for search feature)
CREATE INDEX idx_channels_name_trgm ON channels USING gin (name gin_trgm_ops);

-- Find inactive channels for cleanup
CREATE INDEX idx_channels_last_activity ON channels (last_activity);

-- Rate limiting: count channels per IP
CREATE INDEX idx_channels_creator_ip ON channels (creator_ip);
```

#### JSONB Format: link\_limitation

```json
["dingtalk.com", "weixin.qq.com", "feishu.cn"]
```

Empty array `[]` means all domains are allowed.

***

#### 2. channel\_stats (Materialized View)

Pre-computed channel statistics for fast reads.

```sql
CREATE MATERIALIZED VIEW channel_stats AS
SELECT
    c.id,
    c.name,
    c.channel_type,
    c.password_hash IS NOT NULL AS has_password,
    c.message_count,
    c.last_activity,
    c.created_at
FROM channels c
WITH DATA;

-- Refresh every 5 minutes
-- Or refresh on demand after message operations
```

> **Note**: Since messages are in-memory, `message_count` is maintained as a denormalized counter in the `channels` table and updated on every message insert/delete. The materialized view is optional and mainly for read-heavy list operations.

***

## In-Memory Data Structures

### 1. Message Store

Messages are stored entirely in memory using DashMap for concurrent access.

#### Rust Type Definition

```rust
use dashmap::DashMap;
use std::sync::Arc;
use compact_str::CompactString;
use chrono::{DateTime, Utc};

struct Message {
    id: CompactString,
    name: CompactString,
    link: Arc<str>,
    link_domain: CompactString,
    message_type: Option<CompactString>,
    location: Option<CompactString>,
    expire_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

struct ChannelMessages {
    messages: Arc<DashMap<CompactString, Arc<Message>>>,
    broadcast_tx: tokio::sync::broadcast::Sender<Arc<Message>>,
    subscriber_count: std::sync::atomic::AtomicUsize,
}
```

#### Memory Layout

```
AppState
  └── channels: Arc<DashMap<CompactString, Arc<Channel>>>
        │
        ├── "a1b2c3d4" → Arc<Channel>
        │     ├── id: CompactString ("a1b2c3d4")     ← inline, no heap
        │     ├── name: CompactString ("Math Class")  ← inline, no heap
        │     ├── messages: Arc<DashMap<CompactString, Arc<Message>>>
        │     │     ├── "msg_001" → Arc<Message>
        │     │     │     ├── id: CompactString       ← inline
        │     │     │     ├── name: CompactString     ← inline
        │     │     │     ├── link: Arc<str>          ← shared, zero-copy
        │     │     │     ├── link_domain: CompactString  ← inline
        │     │     │     └── expire_at: CoarseInstant
        │     │     └── "msg_002" → Arc<Message>
        │     ├── broadcast_tx: broadcast::Sender
        │     └── subscriber_count: AtomicUsize
        │
        └── "e5f6g7h8" → Arc<Channel>
              └── ...
```

#### Lifecycle

```
Message Created
    │
    ├── Insert into DashMap
    ├── Broadcast to all subscribers (zero-copy via Arc)
    ├── Increment channel.message_count in PostgreSQL
    │
    │  ... message lives for up to 1 hour ...
    │
    ├── Expire check (every 2 minutes by cleanup task)
    │     └── If expired: remove from DashMap
    │     └── Decrement channel.message_count in PostgreSQL
    │
    └── Or: evicted when channel reaches 300 message limit
          └── Remove oldest message
          └── Decrement channel.message_count in PostgreSQL
```

***

### 2. Rate Limiter Store

```rust
use dashmap::DashMap;
use compact_str::CompactString;
use smallvec::SmallVec;

struct RateLimiter {
    // Key: IP address or client identifier
    // Value: timestamps of recent requests
    requests: DashMap<CompactString, SmallVec<[coarsetime::Instant; 8]>>,
    max_requests: usize,
    window: std::time::Duration,
}
```

#### Cleanup

- Amortized cleanup: old entries removed on each `check()` call
- Background cleanup: every 1 hour, remove stale IP entries

***

### 3. WebSocket Subscriber Tracking

```rust
struct Channel {
    broadcast_tx: tokio::sync::broadcast::Sender<Arc<Message>>,
    subscriber_count: std::sync::atomic::AtomicUsize,
}
```

- `broadcast_tx`: Each subscriber holds a `broadcast::Receiver`
- When receiver is dropped (disconnect), subscriber is automatically removed
- `subscriber_count`: Atomic counter, incremented on connect, decremented on disconnect

***

## Data Flow

### 1. Create Channel Flow

```
Client POST /api/channels
    │
    ├── Validate input (name, password, link_limitation)
    ├── Check rate limit (10 channels per IP)
    ├── Generate 8-char channel ID
    ├── Hash password with bcrypt (if provided)
    │
    ├── INSERT INTO channels (id, name, password_hash, ...)
    │   VALUES ($1, $2, $3, ...)
    │
    ├── Create in-memory Channel struct
    │   ├── DashMap for messages
    │   ├── broadcast channel (buffer: 256)
    │   └── subscriber_count = 0
    │
    └── Return channel info to client
```

### 2. Send Message Flow

```
Client POST /api/channels/:id/messages
    │
    ├── Validate input (name, link, size ≤ 5KB)
    ├── Check link format (http:// or https://)
    ├── Check link domain against channel.link_limitation
    ├── Check rate limit (60 per minute per IP)
    │
    ├── Create Arc<Message> in memory
    │   ├── Generate message ID
    │   ├── Extract link_domain from URL
    │   └── Set expire_at = now + TTL
    │
    ├── Insert into Channel.messages DashMap
    │   └── If channel has 300 messages, evict oldest
    │
    ├── Broadcast to all subscribers (zero-copy)
    │   └── broadcast_tx.send(Arc::clone(&message))
    │
    ├── UPDATE channels SET message_count = message_count + 1
    │   WHERE id = $1
    │
    └── Return message to client
```

### 3. WebSocket Subscribe Flow

```
Client WS /ws/channels/:id
    │
    ├── Lookup channel in DashMap
    │   └── If not found: check PostgreSQL (might be after restart)
    │       └── If found in DB: reconstruct in-memory Channel
    │       └── If not found in DB: return error
    │
    ├── Check password (if required)
    ├── Subscribe to broadcast_tx
    ├── Increment subscriber_count
    │
    ├── Send "connected" message
    ├── Send current messages (from DashMap)
    │
    ├── Loop: receive broadcast messages
    │   ├── Serialize to JSON
    │   └── Send via WebSocket
    │
    ├── On disconnect:
    │   ├── Decrement subscriber_count
    │   └── Drop broadcast receiver (auto-cleanup)
    │
    └── UPDATE channels SET last_activity = NOW()
```

### 4. Cleanup Flow

```
Every 2 minutes (background task):
    │
    ├── Clean expired messages
    │   ├── For each channel in DashMap:
    │   │   └── messages.retain(|_, msg| msg.expires_at > now)
    │   └── Update message_count in PostgreSQL
    │
    ├── Clean inactive channels (30 days)
    │   ├── DELETE FROM channels WHERE last_activity < NOW() - INTERVAL '30 days'
    │   └── Remove from DashMap
    │
    └── Clean stale rate limiter entries
        └── requests.retain(|_, timestamps| !timestamps.is_empty())
```

***

## PostgreSQL Migrations

### Migration 001: Initial Schema

```sql
-- migrations/001_initial.sql

CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE channels (
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

CREATE INDEX idx_channels_type ON channels (channel_type) WHERE channel_type IS NOT NULL;
CREATE INDEX idx_channels_name_trgm ON channels USING gin (name gin_trgm_ops);
CREATE INDEX idx_channels_last_activity ON channels (last_activity);
CREATE INDEX idx_channels_creator_ip ON channels (creator_ip);
```

### Migration 002: Channel Cleanup Function

```sql
-- migrations/002_cleanup.sql

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
```

### Migration 003: Channel Stats View

```sql
-- migrations/003_stats.sql

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

CREATE UNIQUE INDEX idx_channel_stats_id ON channel_stats (id);

CREATE INDEX idx_channel_stats_type ON channel_stats (channel_type)
    WHERE channel_type IS NOT NULL;
```

***

## sqlx Queries

### Channel Queries

```rust
use sqlx::postgres::PgPool;

// Create channel
sqlx::query!(
    r#"
    INSERT INTO channels (id, name, password_hash, link_limitation, channel_type, location, teacher, creator_ip)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    "#,
    channel_id,
    name,
    password_hash,
    link_limitation as serde_json::Value,
    channel_type,
    location,
    teacher,
    creator_ip
)
.execute(&pool)
.await?;

// Get channel by ID
let channel = sqlx::query!(
    r#"
    SELECT id, name, password_hash, link_limitation, channel_type, location, teacher,
           message_count, last_activity, created_at
    FROM channels
    WHERE id = $1
    "#,
    channel_id
)
.fetch_optional(&pool)
.await?;

// Update channel
sqlx::query!(
    r#"
    UPDATE channels
    SET name = COALESCE($2, name),
        password_hash = COALESCE($3, password_hash),
        link_limitation = COALESCE($4, link_limitation),
        channel_type = COALESCE($5, channel_type),
        location = COALESCE($6, location),
        teacher = COALESCE($7, teacher)
    WHERE id = $1
    "#,
    channel_id,
    name,
    password_hash,
    link_limitation,
    channel_type,
    location,
    teacher
)
.execute(&pool)
.await?;

// Delete channel
sqlx::query!(
    r#"DELETE FROM channels WHERE id = $1"#,
    channel_id
)
.execute(&pool)
.await?;

// Increment message count
sqlx::query!(
    r#"
    UPDATE channels
    SET message_count = message_count + 1,
        last_activity = NOW()
    WHERE id = $1
    "#,
    channel_id
)
.execute(&pool)
.await?;

// Decrement message count
sqlx::query!(
    r#"
    UPDATE channels
    SET message_count = GREATEST(message_count - $2, 0)
    WHERE id = $1
    "#,
    channel_id,
    removed_count
)
.execute(&pool)
.await?;

// Update last activity
sqlx::query!(
    r#"
    UPDATE channels SET last_activity = NOW() WHERE id = $1
    "#,
    channel_id
)
.execute(&pool)
.await?;

// List channels with pagination
let channels = sqlx::query!(
    r#"
    SELECT id, name, channel_type, password_hash IS NOT NULL AS has_password,
           message_count, last_activity, created_at
    FROM channels
    WHERE ($1::varchar IS NULL OR channel_type = $1)
      AND ($2::varchar IS NULL OR name ILIKE '%' || $2 || '%')
    ORDER BY last_activity DESC
    LIMIT $3 OFFSET $4
    "#,
    channel_type,
    search,
    limit,
    offset
)
.fetch_all(&pool)
.await?;

// Count channels per IP (rate limiting)
let count = sqlx::query_scalar!(
    r#"
    SELECT COUNT(*) as "count!" FROM channels WHERE creator_ip = $1
    "#,
    ip_address
)
.fetch_one(&pool)
.await?;

// Cleanup inactive channels
sqlx::query!(
    r#"DELETE FROM channels WHERE last_activity < NOW() - INTERVAL '30 days'"#
)
.execute(&pool)
.await?;
```

***

## Connection Configuration

### sqlx Pool Settings

```rust
use sqlx::postgres::{PgPoolOptions, PgConnectOptions};

let pool = PgPoolOptions::new()
    .max_connections(5)          // Small pool: DB is not the bottleneck
    .min_connections(1)          // Keep 1 connection warm
    .acquire_timeout(Duration::from_secs(3))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect_with(
        PgConnectOptions::new()
            .host(&env::var("DB_HOST").unwrap_or("localhost".into()))
            .port(5432)
            .username(&env::var("DB_USER").unwrap_or("qrcode".into()))
            .password(&env::var("DB_PASSWORD").unwrap_or_default())
            .database(&env::var("DB_NAME").unwrap_or("qrcode_share".into()))
    )
    .await?;
```

### PostgreSQL Configuration

```ini
# postgresql.conf (tuned for small server)

# Connections
max_connections = 20

# Memory (for 2GB total server RAM)
shared_buffers = 128MB
effective_cache_size = 512MB
work_mem = 4MB
maintenance_work_mem = 64MB

# WAL
wal_buffers = 4MB
min_wal_size = 80MB
max_wal_size = 1GB

# Query planning
random_page_cost = 1.1          # SSD optimization
effective_io_concurrency = 200  # SSD optimization

# Logging
log_min_duration_statement = 100  # Log slow queries (>100ms)
```

***

## Backup & Recovery

### Backup Strategy

Since messages are in-memory and short-lived (1 hour), only channel metadata needs backup.

```bash
#!/bin/bash
# backup.sh

# Daily PostgreSQL dump
pg_dump -U qrcode -d qrcode_share -F c -f /backup/qrcode_share_$(date +%Y%m%d).dump

# Keep last 7 days
find /backup -name "qrcode_share_*.dump" -mtime +7 -delete
```

### Recovery

```bash
# Restore from backup
pg_restore -U qrcode -d qrcode_share /backup/qrcode_share_20260430.dump

# After restore, in-memory channels will be reconstructed
# on first WebSocket connection (lazy loading)
```

### Restart Behavior

When the server restarts:

1. PostgreSQL data is preserved (channel metadata survives)
2. In-memory data is lost (messages, subscribers, rate limiter state)
3. Channels are lazily loaded from PostgreSQL on first access
4. Messages are gone (acceptable — they have 1-hour TTL anyway)
5. WebSocket clients auto-reconnect and resubscribe

***

## Data Consistency

### In-Memory ↔ PostgreSQL Sync

| Operation       | In-Memory             | PostgreSQL            | Consistency                             |
| --------------- | --------------------- | --------------------- | --------------------------------------- |
| Create channel  | Insert DashMap        | INSERT                | Strong (write both)                     |
| Delete channel  | Remove DashMap        | DELETE                | Strong (write both)                     |
| Send message    | Insert DashMap        | UPDATE count          | Eventual (count synced periodically)    |
| Message expire  | Remove DashMap        | UPDATE count          | Eventual (count synced on cleanup)      |
| Update channel  | Update DashMap        | UPDATE                | Strong (write both)                     |
| Subscriber join | Increment AtomicUsize | UPDATE last\_activity | Eventual (activity synced periodically) |

### Count Reconciliation

Since `message_count` is denormalized, it may drift from the actual in-memory count. Reconciliation runs every 10 minutes:

```rust
async fn reconcile_message_counts(app_state: &AppState, pool: &PgPool) {
    for entry in app_state.channels.iter() {
        let channel_id = entry.key();
        let actual_count = entry.value().messages.len();
        
        sqlx::query!(
            r#"UPDATE channels SET message_count = $2 WHERE id = $1 AND message_count != $2"#,
            channel_id.as_str(),
            actual_count as i32
        )
        .execute(pool)
        .await
        .ok();
    }
}
```

