#!/usr/bin/env bash
# Deploy QRcode Share Backend
#
# Usage:
#   ./scripts/deploy.sh [OPTIONS]
#
# Options:
#   -e, --env ENV       Environment: production, staging (default: production)
#   -i, --image NAME    Docker image name (default: qrcode-share-backend)
#   -t, --tag TAG       Image tag (default: latest)
#   -n, --no-push       Build only, don't push to registry
#   -h, --help          Show help
#
# This script:
#   1. Runs all tests
#   2. Builds Docker image
#   3. Pushes to registry (optional)
#   4. Deploys with docker-compose

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BACKEND_DIR="$PROJECT_ROOT/qrcode_share_backend"

ENV="production"
IMAGE_NAME="qrcode-share-backend"
IMAGE_TAG="latest"
NO_PUSH=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log_step() {
    echo -e "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

log_success() { echo -e "${GREEN}  ✅ $1${NC}"; }
log_error()   { echo -e "${RED}  ❌ $1${NC}"; }
log_info()    { echo -e "  $1"; }

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -e|--env)     ENV="$2"; shift 2 ;;
        -i|--image)   IMAGE_NAME="$2"; shift 2 ;;
        -t|--tag)     IMAGE_TAG="$2"; shift 2 ;;
        -n|--no-push) NO_PUSH=true; shift ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -e, --env ENV       Environment: production, staging"
            echo "  -i, --image NAME    Docker image name"
            echo "  -t, --tag TAG       Image tag"
            echo "  -n, --no-push       Build only, don't push"
            echo "  -h, --help          Show this help"
            exit 0
            ;;
        *) log_error "Unknown option: $1"; exit 1 ;;
    esac
done

FULL_IMAGE="${IMAGE_NAME}:${IMAGE_TAG}"

log_step "Deploy QRcode Share Backend"
log_info "Environment: $ENV"
log_info "Image: $FULL_IMAGE"

# Step 1: Run tests
log_step "Step 1: Run Tests"

cd "$BACKEND_DIR"

if cargo test --lib --test integration_tests --test websocket_tests 2>&1 | tail -5; then
    log_success "Tests passed"
else
    log_error "Tests failed! Aborting deployment."
    exit 1
fi

# Step 2: Build Docker image
log_step "Step 2: Build Docker Image"

cd "$BACKEND_DIR"

docker build \
    --build-arg RUST_LOG=info \
    -t "$FULL_IMAGE" \
    -t "${IMAGE_NAME}:${ENV}" \
    .

if [ $? -eq 0 ]; then
    log_success "Docker image built: $FULL_IMAGE"
else
    log_error "Docker build failed"
    exit 1
fi

# Show image size
IMAGE_SIZE=$(docker images "$FULL_IMAGE" --format "{{.Size}}")
log_info "Image size: $IMAGE_SIZE"

# Step 3: Push to registry (optional)
if [ "$NO_PUSH" = false ]; then
    log_step "Step 3: Push to Registry"
    docker push "$FULL_IMAGE" 2>&1 || {
        log_error "Push failed. Use --no-push to skip."
        exit 1
    }
    log_success "Image pushed: $FULL_IMAGE"
else
    log_step "Step 3: Push Skipped (--no-push)"
fi

# Step 4: Deploy
log_step "Step 4: Deploy with Docker Compose"

cd "$PROJECT_ROOT"

COMPOSE_FILE="docker-compose.yml"
if [ "$ENV" = "staging" ]; then
    COMPOSE_FILE="docker-compose.staging.yml"
fi

if [ -f "$COMPOSE_FILE" ]; then
    # Pull latest images
    docker-compose -f "$COMPOSE_FILE" pull 2>/dev/null || true

    # Start services
    docker-compose -f "$COMPOSE_FILE" up -d

    log_success "Services started"

    # Wait for health check
    log_info "Waiting for services to be healthy..."
    sleep 5

    # Check health
    max_retries=10
    retry=0
    while [ $retry -lt $max_retries ]; do
        if curl -s http://localhost:3000/health > /dev/null 2>&1; then
            log_success "Backend is healthy"
            break
        fi
        retry=$((retry + 1))
        sleep 2
    done

    if [ $retry -eq $max_retries ]; then
        log_error "Backend health check failed"
        docker-compose -f "$COMPOSE_FILE" logs backend
        exit 1
    fi
else
    log_warning "No $COMPOSE_FILE found. Skipping docker-compose deploy."
    log_info "Run manually: docker-compose up -d"
fi

# Step 5: Summary
log_step "Deployment Summary"

echo ""
log_success "Deployment complete!"
log_info "Environment: $ENV"
log_info "Image: $FULL_IMAGE"
log_info "Size: $IMAGE_SIZE"
echo ""

if [ "$ENV" = "production" ]; then
    log_info "Backend: http://localhost:3000"
    log_info "Health:  http://localhost:3000/health"
    log_info "Metrics: http://localhost:3000/metrics"
fi
