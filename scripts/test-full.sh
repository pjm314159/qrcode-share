#!/usr/bin/env bash
# Full integration test pipeline for QRcode Share Backend
#
# Usage:
#   ./scripts/test-full.sh [OPTIONS]
#
# Options:
#   -v, --verbose       Show verbose output
#   -s, --skip-build    Skip build step
#   -k, --keep          Keep containers after tests
#   -h, --help          Show help
#
# This script:
#   1. Builds the project
#   2. Starts PostgreSQL in Docker
#   3. Runs database migrations
#   4. Runs all tests (unit + integration + migration + docker)
#   5. Cleans up Docker containers

set -euo pipefail

# Configuration
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BACKEND_DIR="$PROJECT_ROOT/qrcode_share_backend"
VERBOSE=false
SKIP_BUILD=false
KEEP_CONTAINERS=false
POSTGRES_CONTAINER="qrcode_share_test_postgres"
POSTGRES_PORT=5433
POSTGRES_USER="test_user"
POSTGRES_PASSWORD="test_password"
POSTGRES_DB="qrcode_share_test"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Helper functions
log_step() {
    echo -e "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

log_success() {
    echo -e "${GREEN}  ✅ $1${NC}"
}

log_error() {
    echo -e "${RED}  ❌ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}  ⚠️  $1${NC}"
}

log_info() {
    echo -e "  $1"
}

cleanup() {
    if [ "$KEEP_CONTAINERS" = false ]; then
        log_step "Step 6: Cleanup"
        docker rm -f "$POSTGRES_CONTAINER" 2>/dev/null || true
        log_success "Docker containers removed"
    else
        log_warning "Keeping containers (use -k flag)"
        log_info "PostgreSQL running on localhost:$POSTGRES_PORT"
    fi
}

trap cleanup EXIT

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -s|--skip-build)
            SKIP_BUILD=true
            shift
            ;;
        -k|--keep)
            KEEP_CONTAINERS=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -v, --verbose       Show verbose output"
            echo "  -s, --skip-build    Skip build step"
            echo "  -k, --keep          Keep containers after tests"
            echo "  -h, --help          Show this help"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Check Docker
if ! docker info &> /dev/null; then
    log_error "Docker is not running. Please start Docker."
    exit 1
fi

# Step 1: Build
if [ "$SKIP_BUILD" = false ]; then
    log_step "Step 1: Build Project"

    cd "$BACKEND_DIR"

    if [ "$VERBOSE" = true ]; then
        cargo build 2>&1
    else
        cargo build 2>&1 | tail -5
    fi

    if [ $? -eq 0 ]; then
        log_success "Build succeeded"
    else
        log_error "Build failed"
        exit 1
    fi
fi

# Step 2: Start PostgreSQL
log_step "Step 2: Start PostgreSQL Container"

# Remove existing container if any
docker rm -f "$POSTGRES_CONTAINER" 2>/dev/null || true

# Start PostgreSQL
docker run -d \
    --name "$POSTGRES_CONTAINER" \
    -e POSTGRES_USER="$POSTGRES_USER" \
    -e POSTGRES_PASSWORD="$POSTGRES_PASSWORD" \
    -e POSTGRES_DB="$POSTGRES_DB" \
    -p "$POSTGRES_PORT:5432" \
    postgres:16-alpine

# Wait for PostgreSQL to be ready
log_info "Waiting for PostgreSQL to be ready..."
max_retries=30
retry=0
while [ $retry -lt $max_retries ]; do
    if docker exec "$POSTGRES_CONTAINER" pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB" &>/dev/null; then
        log_success "PostgreSQL is ready"
        break
    fi
    retry=$((retry + 1))
    log_info "Waiting... ($retry/$max_retries)"
    sleep 1
done

if [ $retry -eq $max_retries ]; then
    log_error "PostgreSQL failed to start"
    docker logs "$POSTGRES_CONTAINER"
    exit 1
fi

# Step 3: Run Migrations
log_step "Step 3: Run Database Migrations"

cd "$BACKEND_DIR"

export DATABASE_URL="postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:$POSTGRES_PORT/$POSTGRES_DB"

# Install sqlx-cli if not present
if ! command -v sqlx &> /dev/null; then
    log_info "Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres 2>&1 | tail -3
fi

# Run migrations
if sqlx migrate run --source "$PROJECT_ROOT/migrations"; then
    log_success "Migrations applied successfully"
else
    log_error "Migration failed"
    exit 1
fi

# Verify migrations
log_info "Verifying migration status..."
sqlx migrate info --source "$PROJECT_ROOT/migrations"

# Step 4: Run Tests
log_step "Step 4: Run Tests"

cd "$BACKEND_DIR"

export RUST_BACKTRACE=1
export RUST_LOG="${RUST_LOG:-info}"

TEST_RESULT=0

# 4a: Unit tests
log_info "Running unit tests..."
if cargo test --lib 2>&1 | tail -5; then
    log_success "Unit tests passed"
else
    log_error "Unit tests failed"
    TEST_RESULT=1
fi

# 4b: Integration tests (no Docker needed)
log_info "Running integration tests..."
if cargo test --test integration_tests --test websocket_tests 2>&1 | tail -10; then
    log_success "Integration tests passed"
else
    log_error "Integration tests failed"
    TEST_RESULT=1
fi

# 4c: Migration tests (Docker PostgreSQL)
log_info "Running migration tests..."
if cargo test --test migration_tests 2>&1 | tail -10; then
    log_success "Migration tests passed"
else
    log_error "Migration tests failed"
    TEST_RESULT=1
fi

# 4d: Docker integration tests (testcontainers)
log_info "Running Docker integration tests..."
if cargo test --test docker_tests 2>&1 | tail -10; then
    log_success "Docker integration tests passed"
else
    log_error "Docker integration tests failed"
    TEST_RESULT=1
fi

# Step 5: Summary
log_step "Step 5: Test Summary"

if [ $TEST_RESULT -eq 0 ]; then
    log_success "All tests passed!"
    echo ""
    echo -e "${GREEN}  ╔══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}  ║       ALL TESTS PASSED ✅            ║${NC}"
    echo -e "${GREEN}  ╚══════════════════════════════════════╝${NC}"
else
    log_error "Some tests failed!"
    echo ""
    echo -e "${RED}  ╔══════════════════════════════════════╗${NC}"
    echo -e "${RED}  ║       SOME TESTS FAILED ❌           ║${NC}"
    echo -e "${RED}  ╚══════════════════════════════════════╝${NC}"
fi

exit $TEST_RESULT
