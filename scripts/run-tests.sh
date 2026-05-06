#!/bin/bash

# Run integration tests for QRcode Share Backend
#
# Usage:
#   ./scripts/run-tests.sh [OPTIONS]
#
# Options:
#   -m, --mode MODE          Test mode: testcontainers (default) or docker-compose
#   -t, --test FILTER        Test filter to run specific tests
#   -n, --no-cleanup         Keep Docker containers running after tests
#   -v, --verbose            Show verbose output
#   -h, --help               Show this help message

set -e

# Configuration
MODE="testcontainers"
TEST_FILTER=""
NO_CLEANUP=false
VERBOSE=false
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BACKEND_DIR="$PROJECT_ROOT/qrcode_share_backend"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Functions
print_header() {
    echo -e "\n${CYAN}========================================${NC}"
    echo -e "${CYAN} $1${NC}"
    echo -e "${CYAN}========================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}$1${NC}"
}

print_error() {
    echo -e "${RED}$1${NC}"
}

print_warning() {
    echo -e "${YELLOW}$1${NC}"
}

check_docker() {
    if ! docker info &> /dev/null; then
        print_error "Docker is not running. Please start Docker Desktop."
        exit 1
    fi
}

start_docker_compose() {
    print_header "Starting Docker Compose Test Environment"

    cd "$PROJECT_ROOT"
    docker-compose -f docker-compose.test.yml down -v 2>/dev/null || true
    docker-compose -f docker-compose.test.yml up -d

    print_warning "Waiting for services to be healthy..."
    sleep 10

    # Check health
    local max_retries=30
    local retry=0
    while [ $retry -lt $max_retries ]; do
        if curl -s "http://localhost:3001/health" > /dev/null 2>&1; then
            print_success "Backend is healthy!"
            return 0
        fi
        retry=$((retry + 1))
        print_warning "Waiting for backend... ($retry/$max_retries)"
        sleep 2
    done

    print_error "Backend failed to start within timeout"
    return 1
}

stop_docker_compose() {
    print_header "Stopping Docker Compose Test Environment"

    cd "$PROJECT_ROOT"
    docker-compose -f docker-compose.test.yml down -v --remove-orphans
    print_success "Test environment stopped and cleaned up"
}

run_testcontainers_tests() {
    print_header "Running Tests with Testcontainers"

    cd "$BACKEND_DIR"

    export RUST_LOG="${VERBOSE:-false}" && [ "$VERBOSE" = true ] && RUST_LOG="debug" || RUST_LOG="info"
    export RUST_BACKTRACE=1

    local test_args="test --test docker_tests"
    if [ -n "$TEST_FILTER" ]; then
        test_args="$test_args $TEST_FILTER"
    fi
    if [ "$VERBOSE" = true ]; then
        test_args="$test_args -- --nocapture"
    fi

    cargo $test_args
    return $?
}

run_docker_compose_tests() {
    print_header "Running Tests against Docker Compose Environment"

    cd "$BACKEND_DIR"

    export DATABASE_URL="postgres://test_user:test_password@localhost:5433/qrcode_share_test"
    export RUST_LOG="${VERBOSE:-false}" && [ "$VERBOSE" = true ] && RUST_LOG="debug" || RUST_LOG="info"
    export RUST_BACKTRACE=1

    local test_args="test --test integration_tests --test websocket_tests"
    if [ -n "$TEST_FILTER" ]; then
        test_args="$test_args $TEST_FILTER"
    fi
    if [ "$VERBOSE" = true ]; then
        test_args="$test_args -- --nocapture"
    fi

    cargo $test_args
    return $?
}

show_help() {
    cat << EOF
Run integration tests for QRcode Share Backend

Usage: $0 [OPTIONS]

Options:
  -m, --mode MODE          Test mode: testcontainers (default) or docker-compose
  -t, --test FILTER        Test filter to run specific tests
  -n, --no-cleanup         Keep Docker containers running after tests
  -v, --verbose            Show verbose output
  -h, --help               Show this help message

Examples:
  $0                                    # Run all tests with testcontainers
  $0 -m docker-compose                  # Run tests with docker-compose
  $0 -t test_message_flow               # Run specific test
  $0 -v -n                              # Verbose mode, no cleanup
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -m|--mode)
            MODE="$2"
            shift 2
            ;;
        -t|--test)
            TEST_FILTER="$2"
            shift 2
            ;;
        -n|--no-cleanup)
            NO_CLEANUP=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Main execution
print_header "QRcode Share Integration Tests"
echo -e "Mode: ${MODE}"
echo -e "Project Root: ${PROJECT_ROOT}"

check_docker

success=false

cleanup() {
    if [ "$MODE" = "docker-compose" ] && [ "$NO_CLEANUP" = false ]; then
        stop_docker_compose
    fi
}

trap cleanup EXIT

if [ "$MODE" = "docker-compose" ]; then
    if ! start_docker_compose; then
        print_error "Failed to start Docker Compose environment"
        exit 1
    fi
    run_docker_compose_tests && success=true || true
else
    run_testcontainers_tests && success=true || true
fi

if [ "$success" = true ]; then
    print_header "All Tests Passed!"
    print_success "Status: SUCCESS"
    exit 0
else
    print_header "Some Tests Failed"
    print_error "Status: FAILURE"
    exit 1
fi
