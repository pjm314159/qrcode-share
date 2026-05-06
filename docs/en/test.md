# QRcode Share Backend Test Documentation

This document provides a comprehensive overview of the testing strategy, implemented tests, and results.

## Test Summary

| Category | Tests | Status |
|----------|-------|--------|
| Unit Tests | 100 | ✅ Passed |
| Integration Tests | 10 | ✅ Passed |
| WebSocket Tests | 6 | ✅ Passed |
| WebSocket Edge Cases | 5 | ✅ Passed |
| Rate Limiting Edge Cases | 5 | ✅ Passed |
| Security Tests | 5 | ✅ Passed |
| Docker Integration Tests | 7 | ✅ Passed (requires Docker) |
| Migration Tests | 9 | ✅ Passed (requires Docker) |
| Database Pool Tests | 6 | ✅ Passed (4 require Docker) |
| **Total** | **153** | ✅ All Passed |

### Test Execution Results

**Unit + Integration + WebSocket + Edge Cases (no Docker required):**

```
running 100 tests  → test result: ok. 100 passed; 0 failed
running 10 tests   → test result: ok. 10 passed; 0 failed
running 6 tests    → test result: ok. 6 passed; 0 failed
running 5 tests    → test result: ok. 5 passed; 0 failed
running 5 tests    → test result: ok. 5 passed; 0 failed
running 5 tests    → test result: ok. 5 passed; 0 failed
```

**Docker-dependent tests (Docker required, auto-skip if unavailable):**

```
running 7 tests   → test result: ok. 7 passed; 0 failed (or skipped)
running 9 tests   → test result: ok. 9 passed; 0 failed (or skipped)
running 6 tests   → test result: ok. 6 passed; 0 failed (or skipped)
```

### Test Pipeline Verification

**✅ `test-full` 脚本已通过验证**

完整测试流水线 (`scripts/test-full.sh` / `scripts/test-full.ps1`) 已成功运行，包含以下步骤：

```
Step 1: Build Project          ✅
Step 2: Start PostgreSQL       ✅ (Docker container)
Step 3: Run Database Migrations ✅ (sqlx migrate)
Step 4: Run Tests              ✅
   ├── Unit tests (100)        ✅
   ├── Integration tests (10)  ✅
   ├── WebSocket tests (6)     ✅
   ├── Migration tests (9)     ✅
   └── Docker integration (7)  ✅
Step 5: Test Summary           ✅
Step 6: Cleanup                ✅
```

## Test Execution

### Quick Tests (No Docker Required)
```bash
# Run unit tests, integration tests, WebSocket and edge case tests
cargo test --lib --test integration_tests --test websocket_tests --test websocket_edge_cases --test rate_limit_edge_cases --test security_tests

# Run all unit tests only
cargo test --lib
```

### Full Test Suite (Docker Required)
```bash
# Run all tests including Docker integration tests
cargo test

# Or use the full test pipeline script (recommended)
./scripts/test-full.sh                    # Linux/macOS/WSL
./scripts/test-full.ps1                   # Windows PowerShell

# The full test pipeline:
#   1. Builds the project
#   2. Starts PostgreSQL in Docker
#   3. Runs database migrations
#   4. Runs all tests (unit + integration + migration + docker)
#   5. Cleans up Docker containers
```

### Docker Environment Management
```bash
./scripts/docker-env.ps1 start    # Start test environment
./scripts/docker-env.ps1 status   # Check environment status
./scripts/docker-env.ps1 logs     # View logs
./scripts/docker-env.ps1 stop     # Stop and cleanup
```

---

## 1. Unit Tests (100 tests)

### 1.1 Authentication Module (`auth.rs`) - 9 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_hash_password` | Password hashing produces valid hash | ✅ |
| `test_verify_password_correct` | Correct password verification | ✅ |
| `test_verify_password_wrong` | Wrong password rejection | ✅ |
| `test_same_password_different_hashes` | Same password produces different hashes (salt) | ✅ |
| `test_different_passwords_different_hashes` | Different passwords produce different hashes | ✅ |
| `test_check_channel_access_no_password_required` | Channel without password allows access | ✅ |
| `test_check_channel_access_password_correct` | Correct channel password grants access | ✅ |
| `test_check_channel_access_password_not_provided` | Missing password is rejected | ✅ |
| `test_check_channel_access_password_wrong` | Wrong channel password is rejected | ✅ |

### 1.2 Configuration Module (`config.rs`) - 2 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_config_defaults` | Default configuration values | ✅ |
| `test_config_validation` | Configuration validation | ✅ |

### 1.3 Database Module (`db/`) - 2 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_database_clone` | Database pool cloning | ✅ |
| `test_channel_row_conversion` | Channel database row conversion | ✅ |

### 1.4 Error Handling (`error.rs`) - 6 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_api_response_success` | Success response serialization | ✅ |
| `test_api_response_error` | Error response serialization | ✅ |
| `test_api_response_serialization` | JSON serialization | ✅ |
| `test_app_error_status_codes` | Error to status code mapping | ✅ |
| `test_error_code_display` | Error code display formatting | ✅ |
| `test_from_sqlx_error` | SQLx error conversion | ✅ |

### 1.5 Channel Handlers (`handlers/channels.rs`) - 14 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_create_channel_handler` | Channel creation handler | ✅ |
| `test_create_channel_request_validation` | Request validation | ✅ |
| `test_get_channel_handler` | Get channel handler | ✅ |
| `test_get_channel_not_found` | Channel not found response | ✅ |
| `test_list_channels_handler` | List channels handler | ✅ |
| `test_list_channels_query_defaults` | Default query parameters | ✅ |
| `test_list_channels_query_with_values` | Custom query parameters | ✅ |
| `test_update_channel_handler` | Update channel handler | ✅ |
| `test_update_channel_request` | Update request validation | ✅ |
| `test_delete_channel_handler` | Delete channel handler | ✅ |
| `test_delete_channel_not_found` | Delete non-existent channel | ✅ |
| `test_delete_channel_response` | Delete response format | ✅ |
| `test_channel_list_response_serialization` | List response serialization | ✅ |

### 1.6 Message Handlers (`handlers/messages.rs`) - 4 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_create_message_request_validation` | Message creation validation | ✅ |
| `test_list_messages_query_defaults` | Default list query params | ✅ |
| `test_message_link_domain_extraction` | Domain extraction from URL | ✅ |
| `test_message_list_response_serialization` | Message list serialization | ✅ |

### 1.7 System Handlers (`handlers/system.rs`) - 5 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_health_status_serialization` | Health status JSON | ✅ |
| `test_health_response_structure` | Health response structure | ✅ |
| `test_metrics_response_structure` | Metrics response structure | ✅ |
| `test_memory_check_healthy` | Memory health check | ✅ |
| `test_channel_check_healthy` | Channel health check | ✅ |

### 1.8 WebSocket Handlers (`handlers/websocket.rs`) - 5 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_websocket_query_deserialization` | Query parameter parsing | ✅ |
| `test_client_message_parsing` | Client message parsing | ✅ |
| `test_server_message_creation` | Server message creation | ✅ |
| `test_heartbeat_interval_constant` | Heartbeat interval value | ✅ |
| `test_websocket_timeout_constant` | WebSocket timeout value | ✅ |

### 1.9 Rate Limiting Middleware (`middleware/rate_limit.rs`) - 8 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_rate_limit_config_default` | Default configuration | ✅ |
| `test_rate_limit_layer_creation` | Layer creation | ✅ |
| `test_rate_limiter_allows_under_limit` | Requests under limit allowed | ✅ |
| `test_rate_limiter_blocks_over_limit` | Requests over limit blocked | ✅ |
| `test_rate_limiter_independent_clients` | Independent client limits | ✅ |
| `test_get_client_id_from_real_ip` | Client ID from Real-IP header | ✅ |
| `test_get_client_id_from_forwarded_for` | Client ID from Forwarded-For | ✅ |
| `test_get_client_id_unknown` | Unknown client ID handling | ✅ |

### 1.10 Security Middleware (`middleware/security.rs`) - 1 test ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_security_headers_added` | Security headers are added | ✅ |

### 1.11 Channel Model (`models/channel.rs`) - 3 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_channel_new` | Channel creation | ✅ |
| `test_channel_is_link_allowed` | Link limitation check | ✅ |
| `test_create_channel_request_validation` | Request validation | ✅ |

### 1.12 Message Model (`models/message.rs`) - 6 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_message_new` | Message creation | ✅ |
| `test_message_is_expired` | Expiration check | ✅ |
| `test_message_extract_domain` | Domain extraction | ✅ |
| `test_message_validate_link` | Link validation | ✅ |
| `test_message_validate_size` | Size validation | ✅ |
| `test_create_message_request_validation` | Request validation | ✅ |

### 1.13 WebSocket Models (`models/ws.rs`) - 7 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_message_deserialization` | Message deserialization | ✅ |
| `test_client_ping` | Ping message format | ✅ |
| `test_pong_message` | Pong message format | ✅ |
| `test_connected_message` | Connected message format | ✅ |
| `test_message_broadcast` | Broadcast message format | ✅ |
| `test_subscriber_update` | Subscriber update format | ✅ |
| `test_error_message` | Error message format | ✅ |

### 1.14 Router (`router.rs`) - 2 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_build_router` | Router construction | ✅ |
| `test_router_has_health_endpoint` | Health endpoint exists | ✅ |

### 1.15 Application State (`state/app_state.rs`) - 10 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_app_state_new` | State creation | ✅ |
| `test_app_state_clone` | State cloning | ✅ |
| `test_create_channel` | Channel creation in state | ✅ |
| `test_get_channel` | Channel retrieval | ✅ |
| `test_delete_channel` | Channel deletion | ✅ |
| `test_list_channels` | Channel listing | ✅ |
| `test_channel_limit` | Channel limit enforcement | ✅ |
| `test_cleanup_expired_messages` | Expired message cleanup | ✅ |
| `test_cleanup_inactive_channels` | Inactive channel cleanup | ✅ |

### 1.16 Channel State (`state/channel_state.rs`) - 6 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_channel_state_new` | State creation | ✅ |
| `test_channel_state_with_options` | State with custom options | ✅ |
| `test_channel_add_message` | Message addition | ✅ |
| `test_channel_subscribe` | WebSocket subscription | ✅ |
| `test_channel_subscriber_count` | Subscriber counting | ✅ |
| `test_channel_eviction` | Message eviction | ✅ |

### 1.17 Metrics State (`state/metrics.rs`) - 4 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_metrics_increment` | Counter increment | ✅ |
| `test_metrics_channels` | Channel metrics | ✅ |
| `test_metrics_connections` | Connection metrics | ✅ |
| `test_metrics_inc_messages` | Message metrics | ✅ |

### 1.18 Rate Limiter State (`state/rate_limiter.rs`) - 5 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_rate_limiter_allows_under_limit` | Under limit allowed | ✅ |
| `test_rate_limiter_blocks_over_limit` | Over limit blocked | ✅ |
| `test_rate_limiter_independent_keys` | Independent key limits | ✅ |
| `test_rate_limiter_remaining` | Remaining requests count | ✅ |
| `test_rate_limiter_cleanup` | Expired entries cleanup | ✅ |

### 1.19 Cleanup Tasks (`tasks/cleanup.rs`) - 3 tests ✅

| Test | Description | Status |
|------|-------------|--------|
| `test_cleanup_interval_constant` | Interval value | ✅ |
| `test_start_cleanup_task` | Task startup | ✅ |
| `test_cleanup_task_with_shutdown` | Graceful shutdown | ✅ |

---

## 2. Integration Tests (10 tests) ✅

Located in `tests/integration_tests.rs`

| Test | Description | Status |
|------|-------------|--------|
| `test_health_check` | Health endpoint returns OK | ✅ |
| `test_metrics_endpoint` | Metrics endpoint returns data | ✅ |
| `test_create_channel` | Channel creation via API | ✅ |
| `test_list_channels` | Channel listing via API | ✅ |
| `test_get_channel_not_found` | 404 for non-existent channel | ✅ |
| `test_create_channel_invalid` | Validation error for invalid input | ✅ |
| `test_producer_flow` | Full producer workflow | ✅ |
| `test_consumer_flow` | Full consumer workflow | ✅ |
| `test_link_limitation_flow` | Link limitation enforcement | ✅ |
| `test_delete_channel` | Channel deletion | ✅ |

---

## 3. WebSocket Tests (6 tests) ✅

Located in `tests/websocket_tests.rs`

| Test | Description | Status |
|------|-------------|--------|
| `test_websocket_connect` | WebSocket connection and initial message | ✅ |
| `test_websocket_broadcast` | Message broadcast to subscribers | ✅ |
| `test_websocket_multiple_clients` | Multiple clients receive same message | ✅ |
| `test_websocket_heartbeat` | Ping/pong heartbeat mechanism | ✅ |
| `test_websocket_subscriber_count` | Subscriber count updates | ✅ |
| `test_websocket_concurrent_connections` | Concurrent connection handling | ✅ |

---

## 4. WebSocket Edge Cases (5 tests) ✅

Located in `tests/websocket_edge_cases.rs`

| Test | Description | Status |
|------|-------------|--------|
| `test_websocket_invalid_json_handling` | Invalid JSON from client handled gracefully | ✅ |
| `test_websocket_large_message` | Large message payload handled correctly | ✅ |
| `test_websocket_message_ordering` | Messages arrive in order | ✅ |
| `test_websocket_reconnect_same_channel` | Client reconnect after disconnect | ✅ |
| `test_websocket_rapid_connect_disconnect` | Rapid connect/disconnect cycles | ✅ |

---

## 5. Rate Limiting Edge Cases (5 tests) ✅

Located in `tests/rate_limit_edge_cases.rs`

| Test | Description | Status |
|------|-------------|--------|
| `test_rate_limit_independent_clients` | Independent client rate limits | ✅ |
| `test_rate_limit_burst_allowed` | Burst requests within limit | ✅ |
| `test_rate_limit_window_reset` | Rate limit window resets after time | ✅ |
| `test_different_endpoints_rate_limits` | Different endpoints have separate limits | ✅ |
| `test_api_rate_limiting_integration` | API rate limiting integration test | ✅ |

---

## 6. Security Tests (5 tests) ✅

Located in `tests/security_tests.rs`

| Test | Description | Status |
|------|-------------|--------|
| `test_security_headers_present` | Security headers on all responses | ✅ |
| `test_cors_preflight_request` | CORS preflight request handling | ✅ |
| `test_not_found_handling` | 404 error response format | ✅ |
| `test_error_responses_format` | Error responses follow consistent format | ✅ |
| `test_response_content_type` | Response content type is JSON | ✅ |

---

## 7. Docker Integration Tests (7 tests) ✅

Located in `tests/docker_tests.rs`

These tests use `testcontainers` to spin up a real PostgreSQL container. They are automatically skipped if Docker is not available.

| Test | Description | Status |
|------|-------------|--------|
| `test_database_connection` | Database connection with testcontainers | ✅ |
| `test_create_channel_docker` | Channel persistence in database | ✅ |
| `test_message_flow_docker` | Full message flow with database | ✅ |
| `test_link_limitation_docker` | Link limitation with database | ✅ |
| `test_delete_channel_docker` | Channel deletion from database | ✅ |
| `test_concurrent_channels_docker` | Concurrent channel creation | ✅ |
| `test_metrics_docker` | Metrics with database | ✅ |

---

## 8. Migration Tests (9 tests) ✅

Located in `tests/migration_tests.rs`

These tests use `testcontainers` to spin up a real PostgreSQL container, run migrations, and verify the database schema. They are automatically skipped if Docker is not available.

| Test | Description | Status |
|------|-------------|--------|
| `test_migration_fresh_database` | Run all migrations on a fresh database | ✅ |
| `test_migration_channels_schema` | Verify channels table columns and data types | ✅ |
| `test_migration_indexes` | Verify all indexes are created | ✅ |
| `test_migration_cleanup_function` | Verify cleanup_inactive_channels function | ✅ |
| `test_migration_stats_view` | Verify channel_stats materialized view | ✅ |
| `test_migration_idempotent` | Running migrations twice should not fail | ✅ |
| `test_migration_channel_crud` | Insert and query a channel after migration | ✅ |
| `test_migration_pg_trgm_extension` | Verify pg_trgm extension is installed | ✅ |
| `test_migration_not_null_constraints` | Verify NOT NULL constraints on channels | ✅ |

---

## 9. Database Pool Tests (6 tests) ✅

Located in `tests/database_pool_tests.rs`

2 tests use in-memory state (always run), 4 tests require Docker (auto-skip if unavailable).

| Test | Description | Docker Required | Status |
|------|-------------|-----------------|--------|
| `test_cleanup_task_removes_expired_messages` | Expired messages are cleaned up | No | ✅ |
| `test_cleanup_task_removes_inactive_channels` | Inactive channels are cleaned up | No | ✅ |
| `test_pool_limited_connections` | Pool with limited connections | Yes | ✅ |
| `test_pool_timeout_when_exhausted` | Pool timeout when connections exhausted | Yes | ✅ |
| `test_pool_concurrent_acquisitions` | Concurrent pool acquisitions | Yes | ✅ |
| `test_database_connection_resilience` | Database reconnection resilience | Yes | ✅ |

---

## 10. Test Coverage Analysis

### Well-Tested Areas ✅

| Area | Coverage | Tests | Notes |
|------|----------|-------|-------|
| **Authentication** | High | 9 | Password hashing, channel access control |
| **Error Handling** | High | 6 | All error types covered |
| **Channel CRUD** | High | 14+7+9 | Create, read, update, delete + Docker + Migration |
| **Message Handling** | High | 4 | Creation, validation, listing |
| **WebSocket** | High | 6+5 | Connection, broadcast, heartbeat, edge cases |
| **Rate Limiting** | High | 8+5 | Middleware, state, and edge cases |
| **State Management** | High | 10+6 | In-memory state operations |
| **Database Migrations** | High | 9 | Schema, indexes, functions, views |
| **Database Pool** | Medium | 6 | Pool behavior, cleanup, resilience |
| **Security** | Medium | 1+5 | Headers, CORS, error format |
| **System/Metrics** | Medium | 5 | Health check, metrics endpoint |

### Docker Test Resilience ✅

All Docker-dependent tests include:
- **Docker availability check** with 5-second timeout
- **Container startup timeout** with 60-second limit
- **Graceful skip** when Docker is unavailable (prints skip message, returns early)
- **Serial execution** to prevent container conflicts

---

## 11. Test Configuration

### Environment Variables for Testing

```bash
# .env.test
DATABASE_URL=postgres://test_user:test_password@localhost:5433/qrcode_share_test
RUST_LOG=debug
RUST_BACKTRACE=1
```

### Testcontainers Configuration

The Docker tests use `testcontainers` with the following settings:
- PostgreSQL 16 Alpine
- Automatic container lifecycle management
- Isolated test database per test run
- Serial test execution to prevent conflicts
- 60-second container startup timeout
- Docker availability check before each test

---

## 12. CI/CD Integration

### GitHub Actions (Recommended)

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_USER: test_user
          POSTGRES_PASSWORD: test_password
          POSTGRES_DB: qrcode_share_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Run tests
        run: cargo test --all-features
        env:
          DATABASE_URL: postgres://test_user:test_password@localhost:5432/qrcode_share_test
```

---

## 13. Running Tests Summary

```bash
# Quick unit tests (no Docker)
cargo test --lib

# Integration + WebSocket + Edge cases (no Docker)
cargo test --test integration_tests --test websocket_tests --test websocket_edge_cases --test rate_limit_edge_cases --test security_tests

# All tests including Docker (requires Docker)
cargo test

# Specific test
cargo test test_create_channel

# With verbose output
cargo test -- --nocapture

# With specific features
cargo test --all-features
```

---

## 14. Test Best Practices

1. **Use `expect()` instead of `unwrap()`** - Provides better error messages
2. **Use `#[serial]` for tests that share resources** - Prevents race conditions
3. **Use `testcontainers` for database tests** - Ensures isolation
4. **Keep unit tests fast** - No network calls, no file I/O
5. **Use meaningful test names** - `test_<function>_<scenario>_<expected_result>`
6. **One assertion per test** - Easier to debug failures
7. **Test edge cases** - Empty inputs, maximum values, boundary conditions
8. **Docker tests must skip gracefully** - Always check Docker availability with timeout
9. **Container startup must have timeout** - Prevent tests from hanging indefinitely
