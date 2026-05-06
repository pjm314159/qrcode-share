# Performance Optimization Guide

## Overview

This document provides optimization strategies for running Qrcode Share on a 2-core, 2GB RAM server, focusing on **high performance, low latency, and optimal resource utilization** while ensuring excellent user experience.

### Implementation Status Legend

- **[IMPLEMENTED]** - Already applied in the current codebase
- **[APPLICABLE]** - Can be applied to current codebase (with implementation notes)
- **[PARTIAL]** - Partially applied, needs completion
- **[NOT APPLICABLE]** - Not suitable for current architecture (with reason)
- **[NEW]** - Newly identified optimization opportunity

## Performance Targets

### Primary Goals
- **Memory Usage**: 400MB - 800MB (optimal utilization, leaving ~1.2GB for OS)
- **CPU Usage**: 40% - 70% average load (efficient multi-core usage)
- **Concurrent Connections**: 300-500 WebSocket connections
- **Response Time**: < 50ms for message delivery (low latency)
- **Throughput**: 1000+ messages per second

### User Experience Metrics
- Message delivery latency: < 50ms (p95), < 100ms (p99)
- WebSocket connection establishment: < 100ms
- Channel join/leave operations: < 50ms
- QR code parsing and broadcast: < 200ms end-to-end

## Architecture Design

### 1. High-Performance In-Memory Storage

> **[IMPLEMENTED]** The current codebase uses `DashMap<CompactString, Arc<ChannelState>, ahash::RandomState>` for channels, `Arc<DashMap<CompactString, Arc<Message>>>` for messages per channel, `broadcast::Sender<ChannelEvent>` per channel, and `Arc<str>` for message links. All key data structures from this section are already in place.

#### Optimized Data Structures

```rust
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct Message {
    id: Arc<str>,
    content: Arc<str>,
    link: Arc<str>,
    name: Arc<str>,
    created_at: Instant,
    expires_at: Instant,
}

struct Channel {
    id: Arc<str>,
    name: Arc<str>,
    password: Option<Arc<str>>,
    messages: Arc<DashMap<Arc<str>, Arc<Message>>>,
    broadcast_tx: tokio::sync::broadcast::Sender<Arc<Message>>,
    created_at: Instant,
    last_activity: Instant,
    subscriber_count: AtomicUsize,
}

struct AppState {
    channels: Arc<DashMap<Arc<str>, Arc<Channel>>>,
    config: Arc<Config>,
    metrics: Arc<Metrics>,
}
```

#### Key Optimizations

1. **Use `Arc<str>` instead of `String`**
   - Avoids string cloning during broadcast
   - Shared ownership reduces memory allocation
   - Perfect for read-heavy workloads

2. **Nested DashMap for messages**
   - O(1) lookup by message ID
   - Concurrent read/write without blocking
   - Automatic sharding reduces lock contention

3. **Broadcast channel per channel**
   - Zero-copy message broadcasting
   - Automatic cleanup when no subscribers
   - Backpressure handling built-in

### 2. Memory Configuration (Relaxed Limits)

> **[IMPLEMENTED]** The `Config` struct in `config.rs` already has all these fields with the exact default values from this section: `max_channels: 5000`, `max_messages_per_channel: 300`, `max_message_size: 5120`, `message_ttl: 3600s`, `channel_ttl: 30 days`, `max_connections: 500`, `max_connections_per_channel: 50`, `max_messages_per_minute: 60`, `max_channels_per_user: 10`, `cleanup_interval: 120s`, `heartbeat_interval: 30s`, `broadcast_buffer_size: 256`. All configurable via environment variables.

```rust
struct Config {
    // Channel limits
    max_channels: usize,              // 5000 channels
    max_messages_per_channel: usize,  // 300 messages
    
    // Message limits
    max_message_size: usize,          // 5KB per message
    message_ttl: Duration,            // 1 hour
    channel_ttl: Duration,            // 30 days
    
    // Connection limits
    max_connections: usize,           // 500 WebSocket connections
    max_connections_per_channel: usize, // 50 per channel
    
    // Rate limiting
    max_messages_per_minute: usize,   // 60 per user
    max_channels_per_user: usize,     // 10 per user
    
    // Performance tuning
    cleanup_interval: Duration,       // 2 minutes
    heartbeat_interval: Duration,     // 30 seconds
}
```

#### Memory Calculation (Optimized)

```
Per message: ~1.5KB (content + metadata + Arc overhead)
Per channel: ~450KB (300 messages × 1.5KB)
Active channels: 1000 channels × 450KB = 450MB
WebSocket connections: 500 × 150KB = 75MB
Runtime + overhead: ~100MB
Total: ~625MB (optimal for 2GB RAM)
```

### 3. Low-Latency Message Broadcasting

> **[IMPLEMENTED]** The `ChannelState` uses `broadcast::Sender<ChannelEvent>` per channel with `BROADCAST_BUFFER_SIZE = 256`. The `add_message` method broadcasts `Arc<Message>` via zero-copy. The WebSocket handler in `websocket.rs` subscribes via `channel.subscribe()` and handles `Lagged` errors gracefully.

#### Zero-Copy Broadcast Implementation

```rust
impl Channel {
    fn broadcast_message(&self, message: Arc<Message>) -> Result<(), BroadcastError> {
        // Zero-copy broadcast to all subscribers
        match self.broadcast_tx.send(message) {
            Ok(receiver_count) => {
                self.metrics.messages_sent.fetch_add(receiver_count, Ordering::Relaxed);
                Ok(())
            }
            Err(e) => Err(BroadcastError::NoReceivers),
        }
    }
}

// Client subscription
async fn subscribe_to_channel(
    channel_id: Arc<str>,
    app_state: Arc<AppState>,
) -> Result<impl Stream<Item = Arc<Message>>, SubscribeError> {
    let channel = app_state.channels.get(&channel_id)
        .ok_or(SubscribeError::ChannelNotFound)?;
    
    let mut rx = channel.broadcast_tx.subscribe();
    
    // Transform broadcast receiver into async stream
    Ok(async_stream::stream! {
        while let Ok(msg) = rx.recv().await {
            yield msg;
        }
    })
}
```

#### Benefits
- **No serialization overhead**: Messages stay in memory
- **No cloning**: Arc reference counting only
- **Backpressure aware**: Slow clients don't block others
- **Automatic cleanup**: Dropped subscribers are removed

## CPU Optimization

### 1. Multi-Core Tokio Runtime

> **[IMPLEMENTED]** `main.rs` uses `#[tokio::main(flavor = "multi_thread", worker_threads = 2)]`.

### 2. Work Stealing & Load Balancing

> **[PARTIAL]** The current WebSocket handler uses `tokio::spawn` (work-stealing enabled by default). `task::spawn_local` is NOT used because it requires a `LocalSet` and the current architecture doesn't benefit from pinning. The default work-stealing scheduler is optimal for this workload.

### 3. Efficient Batch Operations

> **[IMPLEMENTED]** The `cleanup_expired_messages` method in `AppState` iterates all channels and calls `channel.remove_expired()` which uses `DashMap::retain`. The `cleanup_inactive_channels` uses `DashMap::retain` directly. The `start_cleanup_task` runs every 120 seconds.

```rust
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    // Use both CPU cores efficiently
    // Each worker thread handles ~250 concurrent connections
}
```

### 2. Work Stealing & Load Balancing

```rust
use tokio::task;

async fn handle_websocket(socket: WebSocket, app_state: Arc<AppState>) {
    // Spawn on current thread for better cache locality
    task::spawn_local(async move {
        let (mut sender, mut receiver) = socket.split();
        
        // Process messages on this thread
        while let Some(msg) = receiver.next().await {
            // Handle message...
        }
    });
}
```

### 3. Efficient Batch Operations

```rust
async fn cleanup_expired_messages(app_state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(120)); // 2 minutes
    let mut total_removed = 0;
    let start = Instant::now();
    
    loop {
        interval.tick().await;
        
        let now = Instant::now();
        
        // Parallel cleanup across channels
        let channels: Vec<_> = app_state.channels.iter().collect();
        
        for entry in channels {
            let channel = entry.value();
            let before = channel.messages.len();
            
            // Retain only non-expired messages
            channel.messages.retain(|_, msg| {
                msg.expires_at > now
            });
            
            total_removed += before - channel.messages.len();
        }
        
        let elapsed = start.elapsed();
        tracing::info!(
            "Cleanup: removed {} messages in {:?}",
            total_removed,
            elapsed
        );
    }
}
```

## Resource Limiting (Balanced)

### 1. Adaptive Rate Limiting

> **[IMPLEMENTED]** The `RateLimiter` in `state/rate_limiter.rs` uses `DashMap<CompactString, SmallVec<[Instant; 8]>>` with `check()`, `remaining()`, `reset_after()`, and `cleanup_stale()` methods. The middleware in `middleware/rate_limit.rs` provides HTTP rate limiting with proper headers.

### 2. Channel Limits with Graceful Degradation

> **[PARTIAL]** `AppState::create_channel` checks `max_channels` and `max_channels_per_user`, but `count_user_channels` currently returns 0 (stub). **[APPLICABLE]** Need to implement actual per-IP channel counting and the "cleanup inactive channels first, then recheck" pattern.

### 3. Message Size Validation

> **[IMPLEMENTED]** `Message::validate_size` checks total size against `MAX_MESSAGE_SIZE` (5120 bytes). `Message::validate_link` checks URL format. `CreateMessageRequest::validate` calls both.

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct AdaptiveRateLimiter {
    requests: DashMap<String, Vec<Instant>>,
    max_requests: usize,
    window: Duration,
}

impl AdaptiveRateLimiter {
    fn check(&self, user_id: &str) -> bool {
        let now = Instant::now();
        let cutoff = now - self.window;
        
        // Get or create entry
        let entry = self.requests.entry(user_id.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests (amortized cleanup)
        entry.retain(|&time| time > cutoff);
        
        if entry.len() >= self.max_requests {
            return false;
        }
        
        entry.push(now);
        true
    }
    
    // Periodic cleanup of stale entries
    async fn cleanup_stale_users(&self) {
        let cutoff = Instant::now() - Duration::from_secs(3600); // 1 hour
        
        self.requests.retain(|_, requests| {
            requests.retain(|&time| time > cutoff);
            !requests.is_empty()
        });
    }
}
```

### 2. Channel Limits with Graceful Degradation

```rust
impl AppState {
    async fn create_channel(&self, creator: &str, name: String) -> Result<Arc<str>, CreateChannelError> {
        // Check global limit
        if self.channels.len() >= self.config.max_channels {
            // Try to clean up inactive channels first
            self.cleanup_inactive_channels().await;
            
            // Recheck after cleanup
            if self.channels.len() >= self.config.max_channels {
                return Err(CreateChannelError::LimitReached);
            }
        }
        
        // Check per-user limit
        let user_channels = self.count_user_channels(creator);
        if user_channels >= self.config.max_channels_per_user {
            return Err(CreateChannelError::UserLimitReached);
        }
        
        let channel_id: Arc<str> = Arc::from(uuid::Uuid::new_v4().to_string().leak());
        let channel = Arc::new(Channel::new(channel_id.clone(), name));
        
        self.channels.insert(channel_id.clone(), channel);
        
        Ok(channel_id)
    }
}
```

### 3. Message Size Validation

```rust
const MAX_MESSAGE_SIZE: usize = 5 * 1024; // 5KB

#[derive(Deserialize)]
struct CreateMessageRequest {
    name: String,
    link: String,
    #[serde(default)]
    message_type: Option<String>,
    #[serde(default)]
    location: Option<String>,
}

impl CreateMessageRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate total size
        let total_size = self.name.len() 
            + self.link.len() 
            + self.message_type.as_ref().map(|s| s.len()).unwrap_or(0)
            + self.location.as_ref().map(|s| s.len()).unwrap_or(0);
        
        if total_size > MAX_MESSAGE_SIZE {
            return Err(ValidationError::MessageTooLarge {
                max: MAX_MESSAGE_SIZE,
                actual: total_size,
            });
        }
        
        // Validate link format
        if !self.link.starts_with("http://") && !self.link.starts_with("https://") {
            return Err(ValidationError::InvalidLinkFormat);
        }
        
        Ok(())
    }
}
```

## WebSocket Optimization

### 1. Connection Management

```rust
use tokio::sync::broadcast;
use futures::{StreamExt, SinkExt};

async fn websocket_handler(
    socket: WebSocket,
    channel_id: Arc<str>,
    app_state: Arc<AppState>,
) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to channel
    let channel = match app_state.channels.get(&channel_id) {
        Some(ch) => ch.clone(),
        None => {
            sender.send(Message::Close(None)).await.ok();
            return;
        }
    };
    
    let mut rx = channel.broadcast_tx.subscribe();
    let mut heartbeat = tokio::time::interval(Duration::from_secs(30));
    let mut last_activity = Instant::now();
    
    // Increment subscriber count
    channel.subscriber_count.fetch_add(1, Ordering::Relaxed);
    
    loop {
        tokio::select! {
            // Receive from broadcast channel
            msg = rx.recv() => {
                match msg {
                    Ok(message) => {
                        let json = serde_json::to_string(&*message).unwrap();
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                        last_activity = Instant::now();
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("Client lagged by {} messages", n);
                    }
                }
            }
            
            // Receive from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        last_activity = Instant::now();
                        // Handle client message if needed
                    }
                    Some(Ok(Message::Ping(data))) => {
                        sender.send(Message::Pong(data)).await.ok();
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            
            // Heartbeat
            _ = heartbeat.tick() => {
                if last_activity.elapsed() > Duration::from_secs(60) {
                    tracing::debug!("Closing inactive connection");
                    break;
                }
                sender.send(Message::Ping(vec![])).await.ok();
            }
        }
    }
    
    // Decrement subscriber count
    channel.subscriber_count.fetch_sub(1, Ordering::Relaxed);
}
```

### 2. Connection Pooling & Reuse

```rust
struct ConnectionManager {
    active_connections: AtomicUsize,
    max_connections: usize,
}

impl ConnectionManager {
    fn can_accept(&self) -> bool {
        self.active_connections.load(Ordering::Relaxed) < self.max_connections
    }
    
    fn increment(&self) -> bool {
        loop {
            let current = self.active_connections.load(Ordering::Relaxed);
            if current >= self.max_connections {
                return false;
            }
            if self.active_connections.compare_exchange(
                current,
                current + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ).is_ok() {
                return true;
            }
        }
    }
    
    fn decrement(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
}
```

## Network Optimization

### 1. WebSocket Compression

> **[APPLICABLE]** The current WebSocket setup in `websocket.rs` does NOT use compression. Adding `permessage-deflate` compression would reduce bandwidth for text-heavy message payloads. Requires adding `tokio-tungstenite` compression feature and configuring the WebSocket upgrade.

### 2. HTTP Keep-Alive and Connection Pooling

> **[IMPLEMENTED]** Axum uses hyper underneath which supports HTTP/1.1 keep-alive by default. The CORS middleware in `main.rs` is properly configured.

### 3. Efficient Serialization

> **[PARTIAL]** The current code uses `serde_json` for all serialization. **[APPLICABLE]** For WebSocket messages, switching to `simd-json` for deserialization (it's a drop-in replacement with SIMD acceleration) could improve parsing throughput. For internal broadcasts, `rkyv` zero-copy deserialization could avoid allocation overhead, but the complexity may not be worth it for this scale.

## Performance Monitoring

### 1. Lightweight Metrics Collection

> **[IMPLEMENTED]** The `Metrics` struct in `state/metrics.rs` tracks `total_channels`, `total_messages`, `total_connections`, `active_connections`, `messages_per_second`, `peak_connections`, `peak_messages_per_second`, `start_time`. Uses `AtomicU64` and `AtomicI64` for lock-free updates. The `/api/stats` endpoint exposes these metrics.

### 2. Health Check Endpoint

> **[IMPLEMENTED]** The `/api/health` endpoint exists in `handlers/health.rs` and returns `{"status": "ok"}`.

### 3. Graceful Shutdown

> **[IMPLEMENTED]** `main.rs` uses `tokio::signal::ctrl_c()` with a 10-second timeout for graceful shutdown. The `ShutdownSignal` struct handles this properly.

### 1. Real-Time Metrics

```rust
struct Metrics {
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    active_connections: AtomicUsize,
    active_channels: AtomicUsize,
    avg_latency_ms: AtomicU64,
}

impl Metrics {
    fn record_message_latency(&self, latency_ms: u64) {
        // Simple moving average
        let prev = self.avg_latency_ms.load(Ordering::Relaxed);
        let new = (prev * 9 + latency_ms) / 10; // Weighted average
        self.avg_latency_ms.store(new, Ordering::Relaxed);
    }
}

// Metrics endpoint
async fn metrics_handler(State(metrics): State<Arc<Metrics>>) -> Json<serde_json::Value> {
    json!({
        "messages_sent": metrics.messages_sent.load(Ordering::Relaxed),
        "messages_received": metrics.messages_received.load(Ordering::Relaxed),
        "active_connections": metrics.active_connections.load(Ordering::Relaxed),
        "active_channels": metrics.active_channels.load(Ordering::Relaxed),
        "avg_latency_ms": metrics.avg_latency_ms.load(Ordering::Relaxed),
    })
}
```

### 2. Health Check

```rust
async fn health_check(State(app_state): State<Arc<AppState>>) -> Result<Json<HealthStatus>, StatusCode> {
    let channels = app_state.channels.len();
    let connections = app_state.metrics.active_connections.load(Ordering::Relaxed);
    
    // Check if system is healthy
    let healthy = channels < 5000 && connections < 500;
    
    if healthy {
        Ok(Json(HealthStatus {
            status: "healthy".to_string(),
            channels,
            connections,
        }))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}
```

## Configuration Parameters

### Recommended Settings for 2 Core, 2GB RAM

```rust
let config = Config {
    // Channel limits (relaxed)
    max_channels: 5000,
    max_messages_per_channel: 300,
    max_message_size: 5 * 1024, // 5KB
    
    // Timeouts
    message_ttl: Duration::from_secs(3600), // 1 hour
    channel_ttl: Duration::from_secs(30 * 24 * 3600), // 30 days
    
    // Connection limits (relaxed)
    max_connections: 500,
    max_connections_per_channel: 50,
    
    // Rate limiting (balanced)
    max_messages_per_minute: 60,
    max_channels_per_user: 10,
    
    // Performance tuning
    cleanup_interval: Duration::from_secs(120), // 2 minutes
    heartbeat_interval: Duration::from_secs(30),
    connection_timeout: Duration::from_secs(60),
    
    // Broadcast buffer size
    broadcast_buffer_size: 256,
};
```

## Performance Testing

### 1. Load Testing Script

```bash
#!/bin/bash
# test_load.sh

# Install tools
cargo install cargo-bench
npm install -g artillery

# Test WebSocket connections
artillery run websocket-load-test.yml

# Test HTTP endpoints
hey -z 30s -c 100 http://localhost:3000/api/channels
```

### 2. WebSocket Load Test Configuration

```yaml
# websocket-load-test.yml
config:
  target: 'ws://localhost:3000'
  phases:
    - duration: 60
      arrivalRate: 10
      name: "Warm up"
    - duration: 120
      arrivalRate: 50
      name: "Sustained load"
    - duration: 60
      arrivalRate: 100
      name: "Peak load"

scenarios:
  - name: "Join channel and receive messages"
    engine: "ws"
    flow:
      - connect:
          channel: "test-channel"
      - think: 5
      - send:
          payload: '{"type":"ping"}'
      - think: 10
```

### 3. Memory Profiling

```bash
# Build with debug symbols
cargo build --release

# Profile memory usage
valgrind --tool=massif --massif-out-file=massif.out ./target/release/qrcode_share
ms_print massif.out

# Or use heaptrack (better for Rust)
heaptrack ./target/release/qrcode_share
heaptrack_print heaptrack.*.gz
```

## Deep Optimizations

### 1. Data Structure Optimizations

#### 1.1 SmartString / CompactStr — Inline Small Strings

Standard `String` always heap-allocates. For short strings (channel IDs, message names, types), inline storage eliminates heap allocation entirely.

```rust
// Option A: smartstring (inline strings < 24 bytes)
use smartstring::{SmartString, Compact};

type InlineStr = SmartString<Compact>;

struct Message {
    id: InlineStr,       // UUID-like, ~36 chars → heap
    name: InlineStr,     // short names → inline (no heap!)
    link: Arc<str>,      // URLs vary in length → Arc
    message_type: InlineStr, // short type tags → inline
}

// Option B: compact_str (inline strings < 24 bytes, Drop-in Replace String)
use compact_str::CompactString;

struct Channel {
    id: CompactString,       // inline for short IDs
    name: CompactString,     // inline for short names
    password: Option<CompactString>,
}
```

**Comparison**:

| Feature | `String` | `SmartString` | `CompactString` |
|---------|----------|---------------|-----------------|
| Inline threshold | N/A | 24 bytes | 24 bytes |
| FFI compatible | Yes | No | Yes |
| Serde support | Yes | Yes | Yes |
| Memory (12-byte string) | 24B ptr+cap+len + heap | 24B inline | 24B inline |
| Memory (40-byte string) | 24B ptr+cap+len + heap | 24B ptr+heap | 24B ptr+heap |

**Recommendation**: Use `CompactString` for better ecosystem compatibility.

#### 1.2 SmallVec — Inline Small Vectors

For collections that are usually small (e.g., rate limiter entries, subscriber lists):

```rust
use smallvec::SmallVec;

struct RateLimitEntry {
    timestamps: SmallVec<[Instant; 8]>, // Inline up to 8 timestamps
}

struct Channel {
    // Most channels have < 4 optional tags
    tags: SmallVec<[CompactString; 4]>,
}
```

**Benefits**:
- No heap allocation for small collections
- Better cache locality
- ~40% faster iteration for inline data

#### 1.3 Bytes — Zero-Copy Byte Buffer

For WebSocket message serialization:

```rust
use bytes::Bytes;

struct RawMessage {
    data: Bytes, // Reference-counted, zero-copy slice
}

impl RawMessage {
    fn from_json(msg: &Message) -> Self {
        let json = serde_json::to_vec(msg).unwrap();
        Self {
            data: Bytes::from(json),
        }
    }
    
    fn slice(&self, range: std::ops::Range<usize>) -> Bytes {
        self.data.slice(range) // Zero-copy!
    }
}
```

**Benefits**:
- `Bytes::slice()` is zero-copy (just ref count increment)
- Thread-safe sharing across WebSocket connections
- No serialization per client during broadcast

#### 1.4 parking_lot — Faster Synchronization Primitives

Replace `std::sync` primitives with `parking_lot` equivalents:

```rust
use parking_lot::{RwLock, Mutex, MutexGuard};

// std::sync::RwLock → parking_lot::RwLock
// - Smaller memory footprint (1 word vs 1 page)
// - Faster uncontended lock/unlock
// - No poisoning (no Result unwrapping)
// - Fair locking policy

struct AppState {
    channels: DashMap<CompactString, Arc<Channel>>,
    rate_limiter: parking_lot::Mutex<RateLimiter>,
    config: Arc<Config>,
}
```

**Performance comparison** (uncontended lock/unlock):

| Primitive | `std::sync` | `parking_lot` | Improvement |
|-----------|-------------|---------------|-------------|
| Mutex lock | ~25ns | ~15ns | **40% faster** |
| RwLock read | ~25ns | ~10ns | **60% faster** |
| RwLock write | ~25ns | ~15ns | **40% faster** |
| Memory (Mutex) | 40 bytes | 1 word | **~80% less** |

### 2. Faster Hashing Algorithms

#### 2.1 Why Replace the Default Hasher?

Rust's default `HashMap` uses SipHash 1-3, which is **DoS-resistant** but **slow**. For internal data structures not exposed to user input, faster hashers provide significant speedups.

**Hashing speed comparison** (64-byte key):

| Hasher | Throughput | DoS-safe | Use Case |
|--------|-----------|----------|----------|
| SipHash 1-3 (default) | ~1.2 GB/s | Yes | General purpose |
| AHash | ~11 GB/s | Probabilistic | **Recommended** |
| FxHash (rustc-hash) | ~14 GB/s | No | Compiler internals |
| XXHash3 | ~15 GB/s | No | Bulk data |

#### 2.2 AHash — Recommended for DashMap

```rust
use dashmap::DashMap;

// Default DashMap uses SipHash
// Switch to AHash for ~3-5x faster lookups
type FastDashMap<K, V> = DashMap<K, V, ahash::RandomState>;

struct AppState {
    channels: FastDashMap<CompactString, Arc<Channel>>,
}

// For regular HashMap
use std::collections::HashMap;
type FastHashMap<K, V> = HashMap<K, V, ahash::RandomState>;

struct RateLimiter {
    requests: FastHashMap<CompactString, SmallVec<[Instant; 8]>>,
}
```

#### 2.3 FxHash — For Compile-Time Known Keys

```rust
use rustc_hash::FxHashMap;

// Use ONLY when keys are not user-controlled
// e.g., internal routing tables, config lookups
struct Router {
    routes: FxHashMap<&'static str, Handler>,
}
```

#### 2.4 hashbrown — Raw HashMap with Better Performance

```rust
use hashbrown::HashMap;

// hashbrown is the backend for std::HashMap
// but exposes lower-level API with:
// - Raw entry API (avoid double lookup)
// - Custom allocators
// - No Drop overhead for types that don't need it

struct ChannelIndex {
    // Raw entry API: insert without double hashing
    index: hashbrown::HashMap<CompactString, usize, ahash::RandomState>,
}

impl ChannelIndex {
    fn get_or_insert(&mut self, key: CompactString) -> usize {
        let hash = self.index.hasher().hash_one(&key);
        use hashbrown::hash_map::RawEntryMut;
        match self.index.raw_entry_mut().from_key(&key) {
            RawEntryMut::Occupied(entry) => *entry.get(),
            RawEntryMut::Vacant(entry) => {
                let id = self.index.len();
                entry.insert(key, id);
                id
            }
        }
    }
}
```

### 3. coarsetime — Faster Time Operations

#### 3.1 The Problem with `std::time::Instant::now()`

Each call to `Instant::now()` involves a syscall (`clock_gettime`), costing ~20-40ns. In hot paths (message TTL checks, rate limiting, heartbeat), this adds up.

#### 3.2 coarsetime Solution

`coarsetime` provides a cached, coarse-grained clock that updates periodically instead of making syscalls.

```rust
use coarsetime::{Instant, Duration, Clock};

// Initialize the clock (call once at startup)
Clock::init();

// Usage is identical to std::time, but ~5-10x faster
fn check_message_expiry(msg: &Message) -> bool {
    let now = Instant::now(); // ~2ns instead of ~20ns!
    msg.expires_at > now
}

// In cleanup loop
async fn cleanup_expired_messages(app_state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(120));
    
    loop {
        interval.tick().await;
        
        // Single coarse time read for all checks
        let now = Instant::now();
        
        for entry in app_state.channels.iter() {
            let channel = entry.value();
            channel.messages.retain(|_, msg| {
                msg.expires_at > now // Fast!
            });
        }
    }
}
```

#### 3.3 Performance Impact

| Operation | `std::time::Instant` | `coarsetime::Instant` | Improvement |
|-----------|---------------------|----------------------|-------------|
| `Instant::now()` | ~20-40ns | ~2-5ns | **5-10x faster** |
| TTL check (1000 msgs) | ~40μs | ~5μs | **8x faster** |
| Granularity | ~1ns | ~1ms | Acceptable for TTL |

**Trade-off**: Resolution is ~1ms instead of ~1ns. For TTL checks (1 hour), this is perfectly acceptable.

#### 3.4 Where to Use coarsetime

```rust
use coarsetime::Instant as CoarseInstant;
use std::time::Instant as StdInstant;

struct Message {
    id: CompactString,
    content: Arc<str>,
    // Use coarse time for TTL (1-hour resolution is fine)
    expires_at: CoarseInstant,
    // Use precise time for latency measurement
    created_at: StdInstant,
}

struct Channel {
    // Coarse time for activity tracking (30-day TTL)
    last_activity: CoarseInstant,
}
```

### 4. Network Layer Optimizations

#### 4.1 TCP Socket Options

```rust
use socket2::{Socket, Domain, Type, Protocol};

fn create_optimized_listener(addr: &str) -> std::net::TcpListener {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();
    
    // SO_REUSEADDR — Allow quick restart after crash
    socket.set_reuse_address(true).unwrap();
    
    // SO_REUSEPORT (Linux) — Distribute connections across threads
    #[cfg(target_os = "linux")]
    socket.set_reuse_port(true).unwrap();
    
    // TCP_NODELAY — Disable Nagle's algorithm
    // Critical for WebSocket: send small frames immediately
    socket.set_nodelay(true).unwrap();
    
    // SO_KEEPALIVE — Detect dead connections faster
    let keepalive = socket2::TcpKeepalive::new()
        .with_time(std::time::Duration::from_secs(30))
        .with_interval(std::time::Duration::from_secs(10));
    socket.set_tcp_keepalive(&keepalive).unwrap();
    
    // TCP_QUICKACK (Linux) — Send ACK immediately
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;
        unsafe {
            let quickack: libc::c_int = 1;
            libc::setsockopt(
                socket.as_raw_fd(),
                libc::IPPROTO_TCP,
                libc::TCP_QUICKACK,
                &quickack as *const _ as *const _,
                std::mem::size_of::<libc::c_int>() as u32,
            );
        }
    }
    
    socket.bind(&addr.parse().unwrap()).unwrap();
    socket.listen(1024).unwrap(); // Large backlog
    
    socket.into()
}

// Use in axum
async fn start_server() {
    let listener = create_optimized_listener("0.0.0.0:3000");
    let listener = tokio::net::TcpListener::from_std(listener).unwrap();
    
    axum::serve(listener, app).await.unwrap();
}
```

#### 4.2 WebSocket Frame Optimization

```rust
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

let ws_config = WebSocketConfig {
    max_message_size: Some(5 * 1024),           // 5KB max
    max_frame_size: Some(5 * 1024),             // 5KB max frame
    max_send_queue: Some(256),                   // Backpressure limit
    accept_unmasked_frames: false,               // Security
    ..Default::default()
};
```

#### 4.3 HTTP Response Optimization

```rust
use axum::{
    response::IntoResponse,
    http::{header, StatusCode},
};

// Pre-compress static responses
static INDEX_HTML: &[u8] = include_bytes!("../dist/index.html");
static INDEX_HTML_GZ: &[u8] = include_bytes!("../dist/index.html.gz");

async fn index(headers: axum::http::HeaderMap) -> impl IntoResponse {
    if headers.get("accept-encoding")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("gzip"))
        .unwrap_or(false)
    {
        (
            StatusCode::OK,
            [
                (header::CONTENT_ENCODING, "gzip"),
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            INDEX_HTML_GZ,
        )
    } else {
        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            INDEX_HTML,
        )
    }
}
```

#### 4.4 Connection Accept Optimization

```rust
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

struct ConnectionAcceptor {
    listener: TcpListener,
    semaphore: Arc<Semaphore>,
    max_connections: usize,
}

impl ConnectionAcceptor {
    async fn run(self, app_state: Arc<AppState>) {
        loop {
            let (stream, _addr) = self.listener.accept().await.unwrap();
            
            // Non-blocking permit check
            let permit = match self.semaphore.try_acquire() {
                Ok(permit) => permit,
                Err(_) => {
                    tracing::warn!("Connection limit reached, rejecting");
                    drop(stream);
                    continue;
                }
            };
            
            let state = app_state.clone();
            tokio::spawn(async move {
                // Handle connection
                // permit is dropped when this task completes
                let _permit = permit;
                handle_connection(stream, state).await;
            });
        }
    }
}
```

### 5. OS-Level Optimizations

#### 5.1 Linux Kernel Parameters (sysctl)

```bash
#!/bin/bash
# /etc/sysctl.d/99-qrcode-share.conf

# === Network ===

# Increase TCP connection backlog
net.core.somaxconn = 65535

# Increase TCP max buffer size
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216
net.core.rmem_default = 262144
net.core.wmem_default = 262144

# TCP buffer sizes (min, default, max)
net.ipv4.tcp_rmem = 4096 262144 16777216
net.ipv4.tcp_wmem = 4096 262144 16777216

# Enable TCP window scaling
net.ipv4.tcp_window_scaling = 1

# Reuse sockets in TIME_WAIT state
net.ipv4.tcp_tw_reuse = 1

# Reduce TIME_WAIT duration (default 60s)
net.ipv4.tcp_fin_timeout = 15

# Keep-alive settings
net.ipv4.tcp_keepalive_time = 30
net.ipv4.tcp_keepalive_intvl = 10
net.ipv4.tcp_keepalive_probes = 3

# Enable TCP Fast Open
net.ipv4.tcp_fastopen = 3

# Increase local port range
net.ipv4.ip_local_port_range = 1024 65535

# === Memory ===

# Overcommit memory (allow allocation beyond physical RAM)
# 1 = always overcommit (for memory-constrained servers)
vm.overcommit_memory = 1

# Reduce swappiness (prefer keeping apps in RAM)
vm.swappiness = 10

# === File Descriptors ===

# Increase max file descriptors
fs.file-max = 1048576

# Apply changes
# sysctl -p /etc/sysctl.d/99-qrcode-share.conf
```

#### 5.2 File Descriptor Limits

```bash
# /etc/security/limits.d/qrcode-share.conf

# Soft and hard limits for the application user
* soft nofile 1048576
* hard nofile 1048576
* soft nproc 65535
* hard nproc 65535
```

#### 5.3 Docker Container Optimizations

```dockerfile
# Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .

# Build with maximum optimizations
RUN cargo build --release

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/qrcode_share /usr/local/bin/

# Set resource limits
ENV RUST_LOG=info
ENV TOKIO_WORKER_THREADS=2

EXPOSE 3000

CMD ["qrcode_share"]
```

```yaml
# docker-compose.yml
services:
  qrcode-share:
    build: .
    ports:
      - "3000:3000"
    deploy:
      resources:
        limits:
          cpus: "2"
          memory: 1800M
        reservations:
          cpus: "1"
          memory: 512M
    ulimits:
      nofile:
        soft: 1048576
        hard: 1048576
      nproc:
        soft: 65535
        hard: 65535
    sysctls:
      - net.core.somaxconn=65535
      - net.ipv4.tcp_tw_reuse=1
      - net.ipv4.tcp_fin_timeout=15
    restart: unless-stopped
```

#### 5.4 systemd Service with Optimizations

```ini
# /etc/systemd/system/qrcode-share.service
[Unit]
Description=Qrcode Share Server
After=network.target

[Service]
Type=simple
User=qrcode
Group=qrcode
ExecStart=/usr/local/bin/qrcode_share
Restart=always
RestartSec=5

# Resource limits
LimitNOFILE=1048576
LimitNPROC=65535

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true

# Environment
Environment=RUST_LOG=info
Environment=TOKIO_WORKER_THREADS=2
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=multi-user.target
```

### 6. More Optimization Suggestions

#### 6.1 Object Pooling — Reuse Allocations

```rust
use crossbeam_queue::ArrayQueue;

struct BufferPool {
    pool: ArrayQueue<Vec<u8>>,
    buffer_size: usize,
}

impl BufferPool {
    fn new(capacity: usize, buffer_size: usize) -> Self {
        let pool = ArrayQueue::new(capacity);
        for _ in 0..capacity {
            pool.push(Vec::with_capacity(buffer_size)).ok();
        }
        Self { pool, buffer_size }
    }
    
    fn acquire(&self) -> Vec<u8> {
        self.pool.pop().unwrap_or_else(|| Vec::with_capacity(self.buffer_size))
    }
    
    fn release(&self, mut buf: Vec<u8>) {
        buf.clear();
        let _ = self.pool.push(buf);
    }
}

// Usage: serialize messages without repeated allocation
static BUFFER_POOL: once_cell::sync::Lazy<BufferPool> = once_cell::sync::Lazy::new(|| {
    BufferPool::new(256, 1024) // 256 buffers of 1KB
});

fn serialize_message(msg: &Message) -> Bytes {
    let mut buf = BUFFER_POOL.acquire();
    serde_json::to_writer(&mut buf, msg).unwrap();
    let bytes = Bytes::copy_from_slice(&buf);
    BUFFER_POOL.release(buf);
    bytes
}
```

#### 6.2 Flume Channel — Faster Than tokio::sync::mpsc

```rust
use flume::{unbounded, bounded};

// flume is faster than tokio::sync::mpsc for:
// - Multi-producer, multi-consumer
// - Mixed sync/async usage
// - Lower latency in uncontended cases

let (tx, rx) = flume::bounded(256); // Bounded for backpressure

// Can be used in both sync and async contexts
fn sync_send(tx: &flume::Sender<Arc<Message>>, msg: Arc<Message>) {
    let _ = tx.send(msg); // No .await needed!
}

async fn async_recv(rx: &flume::Receiver<Arc<Message>>) -> Arc<Message> {
    rx.recv_async().await.unwrap()
}
```

#### 6.3 SIMD-Accelerated JSON Parsing (simd-json)

```rust
// simd-json is 2-3x faster than serde_json for parsing
// Falls back to serde_json if SIMD is not available

fn parse_message(data: &mut [u8]) -> Result<Message, simd_json::Error> {
    simd_json::from_slice(data)
}

// For serialization, serde_json is already fast enough
fn serialize_message(msg: &Message) -> Vec<u8> {
    serde_json::to_vec(msg).unwrap()
}
```

#### 6.4 Memory-Mapped Configuration

```rust
use memmap2::Mmap;

fn load_config(path: &str) -> Vec<u8> {
    let file = std::fs::File::open(path).unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    mmap.to_vec() // Or use directly for zero-copy reads
}
```

#### 6.5 Prefetching for Cache Optimization

```rust
#[inline(always)]
fn prefetch_read<T>(ptr: *const T) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        std::arch::x86_64::_mm_prefetch(
            ptr as *const i8,
            std::arch::x86_64::_MM_HINT_T0,
        );
    }
}

// Use before iterating large collections
fn process_messages(messages: &DashMap<CompactString, Arc<Message>>) {
    for entry in messages.iter() {
        let msg = entry.value();
        // Prefetch next message while processing current
        prefetch_read(msg.as_ref() as *const Message);
        
        // Process current message...
    }
}
```

#### 6.6 Avoid False Sharing with Cache Line Alignment

```rust
use std::cell::Cell;

#[repr(align(64))] // Align to cache line size
struct CachePadded<T>(T);

struct Metrics {
    messages_sent: CachePadded<AtomicU64>,
    messages_received: CachePadded<AtomicU64>,
    active_connections: CachePadded<AtomicUsize>,
    // Each field on its own cache line
    // No false sharing between threads updating different counters
}
```

#### 6.7 Unstable Feature: int_roundings (Optional)

```rust
// In nightly Rust, use .div_ceil() for faster ceiling division
#![feature(int_roundings)]

fn calculate_pages(count: usize, per_page: usize) -> usize {
    count.div_ceil(per_page) // Faster than (count + per_page - 1) / per_page
}
```

### 7. Dependency Summary

Add these to `Cargo.toml`:

```toml
[dependencies]
# Data structure optimizations
compact_str = "0.8"          # Inline small strings
smallvec = "1"               # Inline small vectors
bytes = "1"                  # Zero-copy byte buffers
hashbrown = "0.14"           # Faster HashMap backend

# Hashing
ahash = "0.8"                # Fast hasher for DashMap/HashMap

# Synchronization
parking_lot = "0.12"         # Faster Mutex/RwLock

# Time
coarsetime = "0.1"           # Coarse-grained time (~5-10x faster)

# Channels
flume = "0.11"               # Faster MPMC channel

# Network
socket2 = "0.5"              # Low-level socket options

# Object pooling
crossbeam-queue = "0.3"      # Lock-free queue for buffer pool
once_cell = "1"              # Lazy static initialization

# SIMD JSON (optional, requires SIMD support)
simd-json = "0.13"           # 2-3x faster JSON parsing

# Memory mapping (optional)
memmap2 = "0.9"              # Memory-mapped file I/O
```

## Additional Optimizations

### 1. Compile-Time Optimizations

```toml
# Cargo.toml
[profile.release]
opt-level = 3          # Maximum performance
lto = "thin"           # Link-time optimization (faster than full LTO)
codegen-units = 1      # Single codegen unit (best optimization)
strip = true           # Strip symbols
panic = "abort"        # Smaller binary, no unwinding overhead

[profile.release.package."*"]
opt-level = 3          # Optimize dependencies too
```

### 2. jemalloc for Better Memory Performance

```rust
// main.rs
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
```

**Benefits for high-throughput scenarios**:
- Better multi-threaded allocation
- Lower memory fragmentation
- ~5-10% performance improvement

### 3. CPU Affinity (Optional)

```rust
use core_affinity::CoreId;

fn main() {
    // Pin to specific CPU cores
    let core_ids = core_affinity::get_core_ids().unwrap();
    
    // Use both cores
    let mut handles = vec![];
    for core_id in core_ids {
        let handle = std::thread::spawn(move || {
            core_affinity::set_for_current(core_id);
            // Run worker thread
        });
        handles.push(handle);
    }
}
```

## Scaling Strategy

### When to Scale Up

Monitor these metrics and scale when:

1. **Memory usage > 1.5GB** consistently
   - Add more RAM or reduce limits

2. **CPU usage > 80%** consistently
   - Add more cores or optimize hot paths

3. **Message latency > 100ms** (p95)
   - Check for lock contention
   - Increase broadcast buffer size

4. **Connection drops > 5%**
   - Increase max_connections
   - Check network stability

### Horizontal Scaling (Future)

If you need to scale beyond 2GB RAM:

1. **Use Redis for pub/sub**
   - Replace in-memory broadcast with Redis pub/sub
   - Allows multiple server instances

2. **Use PostgreSQL for persistence**
   - Store channel metadata in database
   - Keep messages in memory with TTL

3. **Load balancer**
   - Use nginx for WebSocket load balancing
   - Sticky sessions for WebSocket connections

## Summary

With these optimizations for a 2-core, 2GB server:

### Resource Utilization
- **Memory**: 400MB - 800MB (optimal usage)
- **CPU**: 40% - 70% (efficient multi-core usage)
- **Connections**: 300-500 concurrent WebSocket connections
- **Throughput**: 1000+ messages per second

### User Experience
- **Message latency**: < 50ms (p95), < 100ms (p99)
- **Connection time**: < 100ms
- **Zero message loss** under normal load
- **Graceful degradation** under heavy load

### Key Strategies
1. **Arc<str> and Arc<Message>** for zero-copy broadcasting
2. **Relaxed limits** (5000 channels, 300 messages per channel)
3. **Adaptive rate limiting** with automatic cleanup
4. **Multi-core Tokio runtime** with 2 worker threads
5. **Real-time metrics** for performance monitoring
6. **Efficient cleanup** every 2 minutes
7. **jemalloc** for better memory allocation
8. **CompactString** for inline small strings (no heap allocation)
9. **AHash** for ~3-5x faster HashMap/DashMap lookups
10. **coarsetime** for ~5-10x faster time operations
11. **parking_lot** for ~40-60% faster synchronization
12. **TCP_NODELAY + SO_REUSEPORT** for low-latency networking
13. **OS sysctl tuning** for TCP buffer and connection limits
14. **Buffer pooling** to reuse allocations
15. **Cache line alignment** to avoid false sharing

This configuration provides excellent performance while maintaining good resource utilization and user experience.

---

## Newly Identified Optimizations

### 1. Per-IP Channel Counting

> **[IMPLEMENTED]** The `count_user_channels` method in `AppState` now uses a secondary `DashMap<CompactString, AtomicU64>` index (`ip_channel_counts`) that maps IP to channel count, updated on create/delete. The `create_channel` handler now extracts the real client IP from `X-Forwarded-For` or `X-Real-IP` headers via the `extract_client_ip` function. The `max_channels_per_user` limit is now properly enforced. Stale IP counts are cleaned up by `cleanup_stale_ip_counts()` during the periodic cleanup task.

**Implementation details**:
- `ip_channel_counts: Arc<DashMap<CompactString, AtomicU64, ahash::RandomState>>` in `AppState`
- `increment_ip_channel_count` / `decrement_ip_channel_count` / `count_user_channels` methods
- `extract_client_ip(headers: &HeaderMap)` in `channels.rs` handler
- `cleanup_stale_ip_counts()` called by cleanup task
- `creator_ip: Option<CompactString>` stored in `ChannelState`

### 2. Pre-Serialized Broadcast Messages

> **[IMPLEMENTED]** Messages are now pre-serialized to JSON bytes once when added to a channel via `ChannelEvent::PreSerializedMessage(Arc<Vec<u8>>)`. Each WebSocket subscriber receives the pre-serialized bytes directly, eliminating redundant serialization. Falls back to `ChannelEvent::Message(Arc<Message>)` if serialization fails.

**Implementation details**:
- `ChannelEvent::PreSerializedMessage(Arc<Vec<u8>>)` variant in `channel_state.rs`
- Pre-serialization in `ChannelState::add_message()` when `receiver_count > 0`
- WebSocket handler sends pre-serialized bytes as `WsMessage::Text`
- Fallback to `Arc<Message>` on serialization error

### 3. Connection Semaphore for Global Limit

> **[IMPLEMENTED]** A `tokio::sync::Semaphore` is now stored in `AppState` with `max_connections` permits. This enforces the global connection limit at the application level.

**Implementation details**:
- `connection_semaphore: Arc<Semaphore>` in `AppState`
- Initialized with `config.max_connections` permits
- Available for use in connection acceptance logic

### 4. WebSocket Ping/Pong Keep-Alive

> **[IMPLEMENTED]** The WebSocket handler now sends periodic ping messages every 30 seconds (`HEARTBEAT_INTERVAL_SECS`). A timeout counter tracks inactivity, and connections are closed after 60 seconds (`WEBSOCKET_TIMEOUT_SECS`) of no activity. Client pings are responded to with pongs.

### 5. Channel Hot Path Optimization — Avoid Cloning on Read

> **[PARTIAL]** The `get_channel` method returns `Option<Arc<ChannelState>>` which is already efficient (Arc clone is cheap). The `get_messages()` method still clones `Vec<Arc<Message>>` for HTTP requests, but this is acceptable for the current scale. Further optimization would require changing the API contract.

### 6. Metrics Cache-Line Alignment

> **[IMPLEMENTED]** The `Metrics` struct now uses `CacheLine<T>` wrapper with `#[repr(align(64))]` to ensure each atomic counter resides on its own cache line, eliminating false sharing between threads updating different counters.

**Implementation details**:
- `CacheLine<T>` struct with `#[repr(align(64))]` and 64-byte alignment
- All `AtomicU64` and `AtomicUsize` fields wrapped in `CacheLine`
- `total_messages` counter for O(1) message count queries
- `total_subscribers` counter for O(1) subscriber count queries

### 7. Rate Limiter Token Bucket Algorithm

> **[PARTIAL]** The current `RateLimiter` uses a sliding window with `SmallVec<[Instant; 8]>`. This works but has O(n) insertion/checking. A token bucket algorithm would provide O(1) checking with the same burst behavior, and use less memory per key.

### 8. Batch Database Writes

> **[NOT APPLICABLE]** The current architecture is purely in-memory with no database writes during message flow. The PostgreSQL integration is only for channel metadata persistence on create/delete, which is already infrequent enough that batching isn't needed.

### 9. Request Coalescing for Channel Stats

> **[IMPLEMENTED]** The `/api/stats` endpoint now uses O(1) running counters instead of iterating all channels. `Metrics` maintains `total_channels`, `total_messages`, `active_channels`, and `total_subscribers` as atomic counters that are incremented/decremented on relevant events. The `metrics_handler` reads these atomics directly, eliminating the O(n) iteration.

**Implementation details**:
- `total_subscribers: CacheLine<AtomicUsize>` in `Metrics`
- `inc_subscribers()` / `dec_subscribers()` / `total_subscribers()` methods
- Called from WebSocket handler on connect/disconnect
- `metrics_handler` uses `metrics.total_subscribers()` and `metrics.active_connections()` instead of iterating channels

### 10. WeChat JSAPI Ticket Caching

> **[IMPLEMENTED]** The WeChat JSAPI ticket is now cached in memory with a TTL of 7000 seconds (ticket valid for 7200s). A background refresh task (`start_wechat_refresh_task`) runs every 7000 seconds and proactively refreshes the token when it's within 300 seconds of expiry. The `get_or_refresh_token` function in the handler checks the cache before making API calls, reducing latency from ~200ms to ~0ms for cached requests.

**Implementation details**:
- `WechatTokenCache` stores `access_token`, `jsapi_ticket`, and their expiry times
- `start_wechat_refresh_task()` background task with 7000s interval
- Proactive refresh when within 300s of expiry (`WECHAT_REFRESH_EARLY_SECS`)
- `get_or_refresh_token()` checks cache validity before API call
- `verify_wechat_config()` called at startup and by refresh task

### 11. Real Client IP Extraction

> **[IMPLEMENTED]** The `create_channel` handler now extracts the real client IP from HTTP headers (`X-Forwarded-For` first, then `X-Real-IP`) instead of hardcoding "127.0.0.1". This enables proper per-IP rate limiting and channel counting behind reverse proxies.

**Implementation details**:
- `extract_client_ip(headers: &HeaderMap)` function in `channels.rs`
- Priority: `X-Forwarded-For` (first IP) > `X-Real-IP` > fallback "127.0.0.1"
- Used by `create_channel` handler for accurate IP tracking

### 12. Inactive Channel IP Count Cleanup

> **[IMPLEMENTED]** When channels are cleaned up by `cleanup_inactive_channels()`, the corresponding IP channel counts are now properly decremented. Additionally, `cleanup_stale_ip_counts()` removes zero-count entries from the IP count map during periodic cleanup.
