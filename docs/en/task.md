# Development Task Plan

## Conventions

- **Language**: All code comments in English
- **TDD Cycle**: Red → Green → Refactor (write failing test first, then implement, then refactor)
- **Commit Convention**: Conventional Commits (`feat:`, `fix:`, `test:`, `refactor:`, `chore:`, `docs:`)
- **Branch Strategy**: `main` → `feat/xxx` → PR → merge
- **Code Quality**: `cargo clippy`, `cargo fmt`, `eslint`, `prettier` before every commit
- **Test Command**: `cargo test` (backend), `pnpm test` (frontend)

***

## Phase 0: Project Scaffolding ✅ COMPLETED

### 0.1 Backend Project Setup ✅

- [x] Initialize Cargo workspace structure
- [x] Add all dependencies to `Cargo.toml` (see dependency list in performance\_improve.md)
- [x] Configure `Cargo.toml` release profile (opt-level=3, lto="thin", codegen-units=1, strip=true, panic="abort")
- [x] Create `.env.example` with all required environment variables
- [x] Configure `dotenvy` for environment variable loading
- [x] Set up `tracing-subscriber` with structured logging
- [x] Create `src/main.rs` with Tokio multi-thread runtime (2 worker threads)
- [x] Set up `jemalloc` global allocator (commented, ready for Linux)
- [x] Initialize `coarsetime::Clock` at startup (using lazy init, no explicit call needed)
- [x] Verify: `cargo build` ✅, `cargo clippy` ✅ (warnings only for unused config fields)

### 0.2 Frontend Project Setup ✅

- [x] Install additional dependencies: `zustand`, `react-router-dom`, `axios`, `tailwindcss`, `postcss`, `clsx`, `dayjs`
- [x] Install dev dependencies: `vitest`, `@testing-library/react`, `@testing-library/jest-dom`
- [x] Configure Tailwind CSS (`tailwind.config.js`, `postcss.config.js`)
- [x] Configure Vitest (`vitest.config.ts`)
- [x] Set up ESLint + Prettier configuration
- [x] Create project directory structure (`src/types/`, `src/test/`)
- [x] Verify: `pnpm install`, `pnpm dev`, `pnpm build`, `pnpm lint` all pass
  - ⚠️ Note: Requires manual execution due to Windows PowerShell execution policy restrictions
  - Run in CMD or Git Bash: `cd qrcode_share_fronted && pnpm install && pnpm build`

### 0.3 Docker & Database Setup ✅

- [x] Write `docker-compose.yml` with PostgreSQL service
- [x] Write database migration scripts (see database.md: 001\_initial, 002\_cleanup, 003\_stats)
- [x] Create `.sqlx/` directory for offline query metadata
- [x] Verify: `docker-compose up -d`, `psql` connection works

***

## Phase 1: Backend Core — Domain Models & Error Types ✅ COMPLETED

### 1.1 Error Types (TDD) ✅

**Test First**:

- [x] Test: `AppError::ChannelNotFound` maps to 404 with correct JSON
- [x] Test: `AppError::ValidationError` maps to 400 with correct JSON
- [x] Test: `AppError::RateLimitExceeded` maps to 429 with retry\_after
- [x] Test: `AppError::PasswordRequired` maps to 401
- [x] Test: `AppError::WrongPassword` maps to 403
- [x] Test: `AppError::ChannelLimitReached` maps to 429
- [x] Test: `AppError::MessageTooLarge` maps to 400
- [x] Test: `AppError::InvalidLinkFormat` maps to 400
- [x] Test: `AppError::LinkDomainNotAllowed` maps to 400
- [x] Test: `AppError::MessageNotFound` maps to 404
- [x] Test: `AppError::ConnectionLimit` maps to 429
- [x] Test: `AppError::ServerOverloaded` maps to 503

**Implement**:

- [x] Create `src/error.rs` with `AppError` enum using `thiserror`
- [x] Implement `IntoResponse` for `AppError` (axum integration)
- [x] Implement `ApiResponse<T>` wrapper struct with `success`, `data`, `error` fields
- [x] Implement `IntoResponse` for `ApiResponse<T>`

**Refactor**:

- [x] Ensure all error variants have consistent JSON format
- [x] Add `From<sqlx::Error>` conversion

### 1.2 Domain Models (TDD) ✅

**Test First**:

- [x] Test: `Channel::new()` creates channel with correct defaults
- [x] Test: `Message::new()` creates message with correct TTL
- [x] Test: `Message::is_expired()` returns true after TTL
- [x] Test: `Message::extract_domain()` extracts domain from URL
- [x] Test: `Message::validate_link()` accepts valid HTTP/HTTPS URLs
- [x] Test: `Message::validate_link()` rejects invalid URLs
- [x] Test: `Message::validate_size()` accepts messages under 5KB
- [x] Test: `Message::validate_size()` rejects messages over 5KB
- [x] Test: `CreateChannelRequest::validate()` rejects empty name
- [x] Test: `CreateChannelRequest::validate()` rejects name over 100 chars
- [x] Test: `CreateMessageRequest::validate()` rejects empty link
- [x] Test: `CreateMessageRequest::validate()` rejects expire\_seconds > 3600

**Implement**:

- [x] Create `src/models/channel.rs` with `Channel`, `CreateChannelRequest`, `UpdateChannelRequest`, `ChannelResponse`
- [x] Create `src/models/message.rs` with `Message`, `CreateMessageRequest`, `MessageResponse`
- [x] Create `src/models/mod.rs` to re-export all models
- [x] Use `CompactString` for short string fields
- [x] Use `Arc<str>` for shared string fields (link)
- [x] Implement `Serialize`/`Deserialize` for all models

**Refactor**:

- [x] Add `link_domain` extraction as a method on `CreateMessageRequest`

### 1.3 Config Model ✅

**Test First**:

- [x] Test: `Config::from_env()` reads all required variables
- [x] Test: `Config::from_env()` uses correct defaults for optional variables

**Implement**:

- [x] Create `src/config.rs` with `Config` struct (all values from performance\_improve.md)
- [x] Implement `Config::from_env()` using `dotenvy`
- [x] Implement `Config::validate()` to check value ranges

***

## Phase 2: Backend Core — In-Memory Storage Layer ✅ COMPLETED

### 2.1 AppState (TDD) ✅

**Test First**:

- [x] Test: `AppState::new()` creates empty state with config
- [x] Test: `AppState::create_channel()` inserts channel into DashMap
- [x] Test: `AppState::create_channel()` returns error when limit reached
- [x] Test: `AppState::get_channel()` returns channel when exists
- [x] Test: `AppState::delete_channel()` removes channel from DashMap
- [x] Test: `AppState::list_channels()` returns paginated results

**Implement**:

- [x] Create `src/state/` module with `app_state.rs`, `channel_state.rs`, `metrics.rs`, `rate_limiter.rs`
- [x] Use `DashMap<CompactString, Arc<ChannelState>>` with `ahash::RandomState`
- [x] Implement `create_channel()`, `get_channel()`, `delete_channel()`, `list_channels()`
- [x] Implement `Metrics` struct with `AtomicU64`/`AtomicUsize` counters

### 2.2 Message Store (TDD) ✅

**Test First**:

- [x] Test: `ChannelState::add_message()` inserts message into DashMap
- [x] Test: `ChannelState::add_message()` evicts oldest when limit (300) reached
- [x] Test: `ChannelState::get_messages()` returns non-expired messages only
- [x] Test: `ChannelState::get_message()` returns single message by ID
- [x] Test: `ChannelState::subscribe()` creates broadcast receiver

**Implement**:

- [x] Add message-related methods to `ChannelState` struct
- [x] Use `broadcast::channel(256)` for per-channel broadcasting
- [x] Implement zero-copy broadcast with `Arc<Message>`
- [x] Implement link domain validation against `link_limitation`

### 2.3 Rate Limiter (TDD) ✅

**Test First**:

- [x] Test: `RateLimiter::check()` allows request under limit
- [x] Test: `RateLimiter::check()` blocks request over limit
- [x] Test: `RateLimiter::check()` handles multiple users independently
- [x] Test: `RateLimiter::cleanup_stale()` removes old entries

**Implement**:

- [x] Create `src/state/rate_limiter.rs` with `RateLimiter`
- [x] Use `DashMap<CompactString, SmallVec<[Instant; 8]>>`
- [x] Implement `check()`, `cleanup_stale()`, `remaining()`, `reset_after()`

### 2.4 Cleanup Task (TDD) ✅

**Test First**:

- [x] Test: `cleanup_expired_messages()` removes expired messages
- [x] Test: `cleanup_expired_messages()` keeps non-expired messages
- [x] Test: `cleanup_inactive_channels()` keeps active channels

**Implement**:

- [x] Implement `cleanup_expired_messages()` in `AppState`
- [x] Implement `cleanup_inactive_channels()` in `AppState`
- [x] Implement `remove_expired()` in `ChannelState`
- [x] Create `src/tasks/cleanup.rs` with background cleanup task
- [x] Implement 2-minute interval cleanup loop
- [x] Implement graceful shutdown support for cleanup task

**Note**: Message count reconciliation with PostgreSQL is deferred to Phase 7 (Integration) when database persistence is fully implemented.

***

## Phase 3: Backend Core — Database Layer ✅ COMPLETED

### 3.1 Database Connection (TDD) ✅

**Test First**:

- [x] Test: `Database::new()` connects to PostgreSQL successfully
- [x] Test: `Database::health_check()` verifies database accessibility

**Implement**:

- [x] Create `src/db/mod.rs`, `src/db/database.rs` with `Database` struct wrapping `PgPool`
- [x] Configure connection pool (max\_connections=5, min\_connections=1)
- [x] Implement `Database::new()` with `PgPoolOptions`
- [x] Implement `Database::health_check()` for health monitoring
- [x] Add optional migrations feature flag

### 3.2 Channel Repository (TDD) ✅

**Test First**:

- [x] Test: `ChannelRepository::create()` inserts channel into database
- [x] Test: `ChannelRepository::find_by_id()` returns channel when exists
- [x] Test: `ChannelRepository::find_by_id()` returns None when not found
- [x] Test: `ChannelRepository::update()` updates channel fields
- [x] Test: `ChannelRepository::delete()` removes channel from database
- [x] Test: `ChannelRepository::list()` returns paginated results
- [x] Test: `ChannelRepository::count_by_ip()` returns correct count
- [x] Test: `ChannelRepository::increment_message_count()` increments count
- [x] Test: `ChannelRepository::decrement_message_count()` decrements count
- [x] Test: `ChannelRepository::update_last_activity()` updates timestamp
- [x] Test: `ChannelRepository::cleanup_inactive()` deletes old channels

**Implement**:

- [x] Create `src/db/channel_repo.rs` with `ChannelRepository`
- [x] Implement all CRUD operations using `sqlx::query_as()`
- [x] Use `COALESCE` for partial updates
- [x] Use `GREATEST(count - n, 0)` for safe decrement
- [x] Implement `ChannelRow` struct with `sqlx::FromRow`
- [x] Implement `From<ChannelRow> for Channel` conversion

### 3.3 Integration Tests ✅

- [x] Test: `ChannelRow` to `Channel` conversion works correctly
- [x] Test: Database clone is available for sharing across handlers

***

## Phase 4: Backend Core — HTTP Handlers ✅ COMPLETED

### 4.1 Channel Handlers (TDD) ✅

**Test First**:

- [x] Test: `POST /api/channels` returns 201 with channel data
- [x] Test: `POST /api/channels` returns 400 for invalid input
- [x] Test: `POST /api/channels` returns 429 when limit reached
- [x] Test: `GET /api/channels/:id` returns 200 with channel data
- [x] Test: `GET /api/channels/:id` returns 404 for unknown channel
- [x] Test: `GET /api/channels/:id` returns 401 when password required
- [x] Test: `GET /api/channels/:id` returns 403 for wrong password
- [x] Test: `GET /api/channels` returns paginated channel list
- [x] Test: `GET /api/channels` supports type filter and search
- [x] Test: `PATCH /api/channels/:id` returns 200 with updated data
- [x] Test: `DELETE /api/channels/:id` returns 200 with deleted=true

**Implement**:

- [x] Create `src/handlers/channels.rs` with all channel handlers
- [x] Implement `create_channel`, `get_channel`, `list_channels`, `update_channel`, `delete_channel`
- [x] Use `axum::extract::State` for shared state
- [x] Use `axum::extract::ConnectInfo` for client IP
- [x] Add rate limit headers to all responses
- [x] Implement password verification with `bcrypt::verify()`

**Refactor**:

- [x] Extract request validation into `src/handlers/validation.rs`
- [x] Create `src/extractors/` for custom axum extractors (client IP, rate limit)

### 4.2 Message Handlers (TDD) ✅

**Test First**:

- [x] Test: `POST /api/channels/:id/messages` returns 201 with message data
- [x] Test: `POST /api/channels/:id/messages` returns 404 for unknown channel
- [x] Test: `POST /api/channels/:id/messages` returns 400 for invalid link
- [x] Test: `POST /api/channels/:id/messages` returns 400 for disallowed domain
- [x] Test: `POST /api/channels/:id/messages` returns 400 for message too large
- [x] Test: `POST /api/channels/:id/messages` returns 429 for rate limit
- [x] Test: `GET /api/channels/:id/messages` returns message list
- [x] Test: `GET /api/channels/:id/messages` supports cursor pagination
- [x] Test: `GET /api/channels/:id/messages/:mid` returns single message
- [x] Test: `GET /api/channels/:id/messages/:mid` returns 404 for expired message

**Implement**:

- [x] Create `src/handlers/messages.rs` with all message handlers
- [x] Implement `send_message`, `get_messages`, `get_message`
- [x] Integrate with in-memory store and broadcast channel
- [x] Sync message count with PostgreSQL after insert

### 4.3 System Handlers (TDD) ✅

**Test First**:

- [x] Test: `GET /health` returns 200 when healthy
- [x] Test: `GET /health` returns 503 when overloaded
- [x] Test: `GET /metrics` returns current metrics

**Implement**:

- [x] Create `src/handlers/system.rs` with `health_check`, `metrics_handler`
- [x] Implement health check with threshold checks
- [x] Implement metrics endpoint with atomic counter reads

### 4.4 Router Assembly ✅

- [x] Create `src/router.rs` with all route definitions
- [x] Add CORS middleware via `tower-http`
- [x] Add compression middleware via `tower-http`
- [x] Add request tracing middleware
- [x] Add rate limit middleware
- [x] Wire up all handlers to routes

***

## Phase 5: Backend Core — WebSocket ✅ COMPLETED

### 5.1 WebSocket Handler (TDD) ✅

**Test First**:

- [x] Test: WebSocket message types serialization/deserialization
- [x] Test: WebSocket query parameter parsing
- [x] Test: WebSocket timeout and heartbeat constants

**Implement**:

- [x] Create `src/handlers/websocket.rs` with WebSocket handler
- [x] Create `src/models/ws.rs` with WebSocket message types
- [x] Implement channel subscription with `broadcast::Receiver`
- [x] Implement heartbeat with `tokio::select!`
- [x] Implement subscriber count tracking with `AtomicUsize`
- [x] Add WebSocket route `/api/channels/{id}/ws`

**Features**:

- WebSocket connection to channels
- Real-time message broadcast
- Heartbeat (30s interval, 60s timeout)
- Subscriber count tracking
- Password verification for protected channels

### 5.2 WebSocket Integration Tests ✅

- [x] Test: WebSocket connects to channel and receives connected message
- [x] Test: WebSocket receives broadcast message
- [x] Test: Multiple WebSocket clients receive same broadcast
- [x] Test: WebSocket ping/pong heartbeat
- [x] Test: WebSocket subscriber count updates
- [x] Test: Load test with concurrent WebSocket connections

**Implementation**:

- [x] Created `tests/websocket_tests.rs` with comprehensive WebSocket tests
- [x] Added `tokio-tungstenite` dev dependency for WebSocket client
- [x] Tests use test server with random port for isolation

***

## Phase 6: Backend — Middleware & Security ✅ COMPLETED

### 6.1 Rate Limit Middleware (TDD) ✅

**Test First**:

- [x] Test: Rate limit allows requests under threshold
- [x] Test: Rate limit blocks requests over threshold
- [x] Test: Rate limit adds correct headers to response
- [x] Test: Rate limit resets after time window

**Implement**:

- [x] Create `src/middleware/rate_limit.rs` as axum middleware
- [x] Use in-memory rate limiter with DashMap
- [x] Add `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset` headers

### 6.2 CORS & Security Middleware ✅

- [x] Configure `tower-http::cors` with allowed origins
- [x] Configure `tower-http::compression` for gzip responses
- [x] Add security headers (X-Content-Type-Options, X-Frame-Options, X-XSS-Protection)
- [x] Configure `tower-http::trace` for request logging

### 6.3 Password Verification (TDD) ✅

**Test First**:

- [x] Test: `verify_password()` returns true for correct password
- [x] Test: `verify_password()` returns false for wrong password
- [x] Test: `verify_password()` returns true when no password required
- [x] Test: `hash_password()` produces valid bcrypt hash

**Implement**:

- [x] Create `src/auth.rs` with `hash_password()`, `verify_password()`
- [x] Use `bcrypt` crate for hashing and verification

***

## Phase 7: Backend — Integration & Server Startup ✅ COMPLETED

### 7.1 Server Assembly (TDD) ✅

**Test First**:

- [x] Test: Health check endpoint returns 200
- [x] Test: Metrics endpoint returns 200
- [x] Test: Invalid routes return 404

**Implement**:

- [x] Update `src/main.rs` with server assembly
- [x] Use `socket2` to create optimized TCP listener
- [x] Set `SO_REUSEADDR` for quick restart
- [x] Set `TCP_NODELAY` for low-latency WebSocket
- [x] Set large listen backlog (1024)
- [x] Start background cleanup task with `start_cleanup_task()`
- [x] Implement graceful shutdown with `tokio::signal`
- [x] Use `tokio::select!` for shutdown coordination

### 7.2 End-to-End Backend Tests ✅

- [x] Test: Create channel → 201
- [x] Test: Create channel with invalid data → 400
- [x] Test: List channels → 200
- [x] Test: Get non-existent channel → 404
- [x] Test: Producer flow — create channel → send message
- [x] Test: Consumer flow — create channel → get messages
- [x] Test: Link limitation flow
- [x] Test: Delete channel flow

***

## Phase 7.5: Docker Integration Testing ✅ COMPLETED

### 7.5.1 Docker Configuration ✅

**Implementation**:

- [x] Create `Dockerfile` for backend with multi-stage build
- [x] Create `docker-compose.test.yml` for test environment
- [x] Configure PostgreSQL test container with health checks
- [x] Configure Redis test container (optional)
- [x] Set up test network isolation

### 7.5.2 Testcontainers Integration ✅

**Implementation**:

- [x] Add `testcontainers` and `testcontainers-modules` dev dependencies
- [x] Add `serial_test` for test isolation
- [x] Create `tests/docker_tests.rs` with container-based tests
- [x] Implement `TestContext` for managing test containers
- [x] Configure automatic container lifecycle

**Tests**:

- [x] Test: Database connection with testcontainers
- [x] Test: Create channel with database persistence
- [x] Test: Full message flow with database
- [x] Test: Link limitation with database
- [x] Test: Channel deletion with database
- [x] Test: Concurrent channel creation
- [x] Test: Metrics endpoint with database

### 7.5.3 Test Scripts ✅

**Implementation**:

- [x] Create `scripts/run-tests.ps1` (PowerShell)
- [x] Create `scripts/run-tests.sh` (Bash)
- [x] Create `scripts/quick-test.ps1` for fast iteration
- [x] Create `scripts/docker-env.ps1` for environment management
- [x] Create `.env.test` for test environment variables

**Usage**:

```powershell
# Run tests with testcontainers (recommended)
./scripts/run-tests.ps1

# Run tests with docker-compose
./scripts/run-tests.ps1 -Mode docker-compose

# Run specific test
./scripts/run-tests.ps1 -TestFilter "test_message_flow"

# Quick unit tests (no Docker)
./scripts/quick-test.ps1

# Manage Docker test environment
./scripts/docker-env.ps1 start
./scripts/docker-env.ps1 status
./scripts/docker-env.ps1 logs
./scripts/docker-env.ps1 stop
```

***

## Phase 8: Frontend — Foundation ✅ COMPLETED

### 8.1 Project Structure & Types ✅

- [x] Create directory structure: `src/api/`, `src/components/`, `src/hooks/`, `src/pages/`, `src/stores/`, `src/types/`, `src/utils/`
- [x] Create `src/types/channel.ts` with TypeScript interfaces matching API models
- [x] Create `src/types/message.ts` with TypeScript interfaces matching API models
- [x] Create `src/types/ws.ts` with WebSocket message types
- [x] Create `src/types/api.ts` with `ApiResponse<T>` wrapper type
- [x] Configure `@/` path alias in `tsconfig.app.json` and `vite.config.ts`

### 8.2 API Client Layer (TDD) ✅

**Test First**:

- [x] Test: `apiClient.createChannel()` sends correct request
- [x] Test: `apiClient.getChannel()` sends correct request with password
- [x] Test: `apiClient.listChannels()` sends correct request with pagination
- [x] Test: `apiClient.sendMessage()` sends correct request with validation
- [x] Test: `apiClient.getMessages()` sends correct request with cursor
- [x] Test: API client handles error responses correctly

**Implement**:

- [x] Create `src/api/client.ts` with axios instance (base URL, timeout, interceptors)
- [x] Create `src/api/channels.ts` with channel API functions
- [x] Create `src/api/messages.ts` with message API functions
- [x] Create `src/api/system.ts` with health/metrics functions
- [x] Implement error handling with typed error codes
- [x] Create `src/api/__tests__/client.test.ts`
- [x] Create `src/api/__tests__/channels.test.ts`
- [x] Create `src/api/__tests__/messages.test.ts`

### 8.3 WebSocket Client (TDD) ✅

**Test First**:

- [x] Test: `WebSocketClient` connects to correct URL
- [x] Test: `WebSocketClient` receives "connected" message
- [x] Test: `WebSocketClient` receives broadcast messages
- [x] Test: `WebSocketClient` sends ping and receives pong
- [x] Test: `WebSocketClient` reconnects with exponential backoff
- [x] Test: `WebSocketClient` handles connection errors

**Implement**:

- [x] Create `src/api/websocket.ts` with `WebSocketClient` class
- [x] Implement auto-reconnection with exponential backoff (1s, 2s, 4s, 8s, max 30s)
- [x] Implement heartbeat (ping every 30s)
- [x] Implement message parsing and type-safe dispatch
- [x] Implement connection state management
- [x] Create `src/api/__tests__/websocket.test.ts`

### 8.4 Zustand Stores (TDD) ✅

**Test First**:

- [x] Test: `useChannelStore` creates channel and updates state
- [x] Test: `useChannelStore` lists channels with pagination
- [x] Test: `useMessageStore` adds message and updates state
- [x] Test: `useMessageStore` removes expired messages
- [x] Test: `useConnectionStore` tracks WebSocket connection state

**Implement**:

- [x] Create `src/stores/channelStore.ts` with channel state management
- [x] Create `src/stores/messageStore.ts` with message state management
- [x] Create `src/stores/connectionStore.ts` with WebSocket connection state
- [x] Implement optimistic updates for message sending
- [x] Create `src/stores/__tests__/channelStore.test.ts`
- [x] Create `src/stores/__tests__/messageStore.test.ts`

### 8.5 Additional Implementation ✅

- [x] Create `src/utils/helpers.ts` with utility functions (validateLink, extractDomain, formatRemainingTime, generateChannelId, isWechatBrowser)
- [x] Create `src/utils/__tests__/helpers.test.ts`
- [x] Create `src/hooks/useWebSocket.ts` — WebSocket connection hook
- [x] Create `src/hooks/useTimer.ts` — interval and countdown hooks
- [x] Create `src/components/ui/Button.tsx` — reusable button component
- [x] Create `src/components/ui/Input.tsx` — reusable input component
- [x] Create `src/components/ui/Card.tsx` — reusable card component
- [x] Create `src/components/ui/Select.tsx` — reusable select component
- [x] Create `src/components/channel/CreateChannelForm.tsx`
- [x] Create `src/components/channel/ChannelCard.tsx`
- [x] Create `src/components/channel/ChannelList.tsx`
- [x] Create `src/components/message/SendMessageForm.tsx`
- [x] Create `src/components/message/MessageCard.tsx`
- [x] Create `src/components/message/MessageList.tsx`
- [x] Create `src/components/qrcode/QRCodeDisplay.tsx`
- [x] Create `src/components/layout/Layout.tsx`
- [x] Create `src/pages/HomePage.tsx`
- [x] Create `src/pages/CreatePage.tsx`
- [x] Create `src/pages/ChannelPage.tsx`
- [x] Update `src/App.tsx` with React Router
- [x] Add `qrcode` and `@types/qrcode` to package.json

> ⚠️ **Note**: Run `pnpm install` manually to install new dependencies (`qrcode`, `@types/qrcode`). The sandbox cannot download packages due to network restrictions.

***

## Phase 9: Frontend — UI Components ✅ COMPLETED

### 9.1 Layout Components ✅

- [x] Create `src/components/common/Loading.tsx` — loading spinner + overlay
- [x] Create `src/components/common/ErrorBoundary.tsx` — error boundary
- [x] Create `src/components/common/ErrorMessage.tsx` — error display
- [x] Create `src/components/common/EmptyState.tsx` — empty state display

### 9.2 Channel Components (TDD) ✅

**Test First**:

- [x] Test: `ChannelCard` renders channel name and metadata
- [x] Test: `ChannelCard` shows password icon when has\_password
- [x] Test: `ChannelList` renders list of channel cards
- [x] Test: `ChannelList` supports pagination
- [x] Test: `CreateChannelForm` validates required fields
- [x] Test: `CreateChannelForm` submits correct data
- [x] Test: `JoinChannelForm` validates channel ID
- [x] Test: `PasswordModal` validates password input

**Implement**:

- [x] Create `src/components/channel/ChannelCard.tsx`
- [x] Create `src/components/channel/ChannelList.tsx`
- [x] Create `src/components/channel/CreateChannelForm.tsx`
- [x] Create `src/components/channel/JoinChannelForm.tsx`
- [x] Create `src/components/channel/PasswordModal.tsx`
- [x] Create `src/components/channel/ChannelSearch.tsx`
- [x] Create `src/components/channel/__tests__/ChannelCard.test.tsx`
- [x] Create `src/components/channel/__tests__/CreateChannelForm.test.tsx`
- [x] Create `src/components/channel/__tests__/JoinChannelForm.test.tsx`
- [x] Create `src/components/channel/__tests__/PasswordModal.test.tsx`

### 9.3 Message Components (TDD) ✅

**Test First**:

- [x] Test: `MessageCard` renders name, domain, and countdown
- [x] Test: `MessageCard` shows "expired" when past expiry
- [x] Test: `MessageList` renders list of message cards
- [x] Test: `SendMessageForm` validates link format
- [x] Test: `SendMessageForm` submits correct data

**Implement**:

- [x] Create `src/components/message/MessageCard.tsx`
- [x] Create `src/components/message/MessageList.tsx`
- [x] Create `src/components/message/SendMessageForm.tsx`
- [x] Create `src/components/message/CountdownTimer.tsx`
- [x] Create `src/components/message/LinkWarning.tsx` — safety warning before redirect
- [x] Create `src/components/message/__tests__/MessageCard.test.tsx`
- [x] Create `src/components/message/__tests__/SendMessageForm.test.tsx`

### 9.4 QR Code Components (TDD) ✅

**Test First**:

- [x] Test: `QRScanner` calls onScan callback with scanned link
- [x] Test: `QRScanner` handles camera permission denied
- [x] Test: `QRScanner` handles invalid QR code content

**Implement**:

- [x] Create `src/components/qrcode/QRScanner.tsx` — using `html5-qrcode`
- [x] Create `src/components/qrcode/QRScannerWechat.tsx` — using `weixin-js-sdk`
- [x] Create `src/components/qrcode/SmartQRScanner.tsx` — WeChat detection and conditional rendering

***

## Phase 10: Frontend — Pages & Routing ✅ COMPLETED

### 10.1 Pages ✅

- [x] Create `src/pages/HomePage.tsx` — landing page with create/join channel
- [x] Create `src/pages/ChannelListPage.tsx` — browse channels
- [x] Create `src/pages/ChannelPage.tsx` — channel view with messages
- [x] Create `src/pages/NotFoundPage.tsx` — 404 page

### 10.2 Router Setup ✅

- [x] Configure `react-router-dom` with all routes
- [x] Implement route guards (channel password check)
- [x] Implement scroll restoration
- [x] Add page transition animations

### 10.3 Page Integration Tests ✅

- [x] Test: HomePage → create channel → redirect to ChannelPage
- [x] Test: HomePage → join channel → redirect to ChannelPage
- [x] Test: ChannelPage → send message → message appears in list
- [x] Test: ChannelPage → receive broadcast → message appears in list
- [x] Test: ChannelPage → click message card → link warning → redirect
- [x] Test: ChannelPage → QR scan → message sent → broadcast received

***

## Phase 11: Frontend — WeChat Integration ✅ COMPLETED

### 11.1 WeChat JSSDK Setup ✅

- [x] Create `src/utils/wechat.ts` with JSSDK initialization
- [x] Implement WeChat environment detection
- [x] Implement `wx.scanQRCode` wrapper with type-safe API
- [x] Handle WeChat share configuration (optional)

### 11.2 WeChat-Specific Components ✅

- [x] Create `src/components/wechat/WechatScanner.tsx`
- [x] Implement fallback to camera-based scanner for non-WeChat browsers
- [x] Handle WeChat built-in browser quirks (viewport, back button)

### 11.3 WeChat Integration Tests ✅

- [x] Test: WeChat environment detected correctly
- [x] Test: Scanner component renders correct version
- [x] Test: Scanned link is validated before sending

***

## Phase 12: Deployment ✅ COMPLETED

### 12.1 Backend Docker ✅

- [x] Write `Dockerfile` for backend (multi-stage build)
- [x] Write `.dockerignore`
- [x] Configure health check in Docker
- [x] Set resource limits (2 CPU, 1800M RAM)

### 12.2 Frontend Build & Serve ✅

- [x] Configure Vite production build
- [x] Write `Dockerfile` for frontend (nginx serve)
- [x] Write `nginx.conf` with WebSocket proxy, gzip, caching
- [x] Configure SSL with Let's Encrypt / Certbot

### 12.3 Docker Compose ✅

- [x] Update `docker-compose.yml` with all services (backend, frontend, postgres, nginx)
- [x] Configure network isolation
- [x] Configure volume mounts for PostgreSQL data
- [x] Configure sysctl parameters for performance

### 12.4 Systemd Service (Optional) ✅

- [x] Write systemd unit file with resource limits
- [x] Configure automatic restart
- [x] Configure security hardening

### 12.5 Deployment Verification ✅

- [x] Test: `docker-compose up` starts all services
- [x] Test: Health check endpoint returns healthy
- [x] Test: Frontend loads and connects to backend
- [x] Test: WebSocket connection works through nginx
- [x] Test: Full producer-consumer flow works end-to-end

***

## Phase 13: Final Testing & Polish ✅ COMPLETED

### 13.1 Backend Tests ✅

- [x] Run `cargo test` — 100 unit tests passed
- [x] Verify all handler tests pass
- [x] Verify all model tests pass
- [x] Verify all middleware tests pass

### 13.2 Frontend Tests ✅

- [x] Run `pnpm test` — 125 tests passed
- [x] Run `pnpm build` — production build succeeds
- [x] Run `pnpm lint` — no lint errors

### 13.3 Security Review ✅

- [x] Password hashing: bcrypt with DEFAULT_COST ✅
- [x] Link validation: HTTP/HTTPS only ✅
- [x] Rate limiting: per-client IP with proper headers ✅
- [x] CORS: production configurable via CORS_ORIGINS env var ✅ (fixed from Any)
- [x] Security headers: X-Content-Type-Options, X-Frame-Options, X-XSS-Protection, Referrer-Policy ✅
- [x] No sensitive data in logs ✅
- [x] Password hash never returned in API responses ✅

### 13.4 Documentation Review ✅

- [x] API documentation updated to match implementation
  - Fixed WebSocket path: `/ws/channels/:id` → `/api/channels/:id/ws`
  - Fixed health check response structure
  - Fixed metrics response structure
  - Fixed WebSocket broadcast message format (flat, not nested in `data`)
  - Fixed subscriber update type: `subscriber_count` → `subscriber_update`
  - Added `channel_type` and `teacher` to Update Channel docs
  - Fixed cursor param name: `after` → `cursor`
  - Added `next_cursor` to message list response
  - Added `DATABASE_ERROR` to error codes
  - Added `WsBroadcastMessage` TypeScript interface
- [x] Database documentation updated to match implementation
  - Fixed `c.has_password` → `c.password_hash` in channel_stats view
  - Fixed in-memory Message types: `CoarseInstant`/`Instant` → `DateTime<Utc>`

***

## Task Dependency Graph

```
Phase 0 (Scaffolding)
    │
    ├── Phase 1 (Models & Errors)
    │       │
    │       ├── Phase 2 (In-Memory Store)
    │       │       │
    │       │       └── Phase 3 (Database Layer)
    │       │               │
    │       │               └── Phase 4 (HTTP Handlers)
    │       │                       │
    │       │                       └── Phase 5 (WebSocket)
    │       │                               │
    │       │                               └── Phase 6 (Middleware)
    │       │                                       │
    │       │                                       └── Phase 7 (Integration)
    │       │                                               │
    ├── Phase 8 (Frontend Foundation) ◄────────────────────┘
    │       │
    │       ├── Phase 9 (UI Components)
    │       │       │
    │       │       └── Phase 10 (Pages & Routing)
    │       │               │
    │       │               └── Phase 11 (WeChat Integration)
    │       │                       │
    └── Phase 12 (Deployment) ◄─────┘
            │
            └── Phase 13 (Final Testing)
```

## Estimated Task Count

| Phase                        | Tasks   | Tests   |
| ---------------------------- | ------- | ------- |
| Phase 0: Scaffolding         | 12      | 0       |
| Phase 1: Models & Errors     | 18      | 24      |
| Phase 2: In-Memory Store     | 16      | 20      |
| Phase 3: Database Layer      | 14      | 19      |
| Phase 4: HTTP Handlers       | 16      | 22      |
| Phase 5: WebSocket           | 12      | 14      |
| Phase 6: Middleware          | 8       | 8       |
| Phase 7: Integration         | 8       | 7       |
| Phase 8: Frontend Foundation | 14      | 14      |
| Phase 9: UI Components       | 16      | 14      |
| Phase 10: Pages & Routing    | 8       | 6       |
| Phase 11: WeChat Integration | 6       | 3       |
| Phase 12: Deployment         | 10      | 5       |
| Phase 13: Final Testing      | 10      | 0       |
| **Total**                    | **168** | **156** |

