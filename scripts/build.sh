#!/bin/bash

# Build script for Argus Events Docker image

set -euo pipefail

# Default values
IMAGE_NAME="argus-events"
TAG="latest"
DOCKERFILE="Dockerfile"
RUN_INTEGRATION_TESTS=true
SKIP_UNIT_TESTS=false
USE_COLOR=false

# Colors (empty by default)
RED=''
GREEN=''
YELLOW=''
NC=''

# Auto-detect colors: TTY + not CI + not NO_COLOR
if [[ -t 1 && "${NO_COLOR:-}" != "1" && "${CI:-}" != "true" ]]; then
    USE_COLOR=true
fi

show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  -t, --tag              Set image tag (default: latest)"
    echo "  -n, --name             Set image name (default: argus-events)"
    echo "  -f, --file             Set Dockerfile path (default: Dockerfile)"
    echo "  --color                Force colored output"
    echo "  --no-color             Disable colored output"
    echo "  --skip-integration     Skip integration tests against container"
    echo "  --skip-unit-tests      Skip local unit tests (Docker still runs them)"
    echo "  -h, --help             Show this help message"
    echo ""
    echo "Colors are auto-detected based on terminal support."
    echo "Set NO_COLOR=1 environment variable to disable colors."
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--tag) TAG="$2"; shift 2 ;;
        -n|--name) IMAGE_NAME="$2"; shift 2 ;;
        -f|--file) DOCKERFILE="$2"; shift 2 ;;
        --color) USE_COLOR=true; shift ;;
        --no-color) USE_COLOR=false; shift ;;
        --skip-integration) RUN_INTEGRATION_TESTS=false; shift ;;
        --skip-unit-tests) SKIP_UNIT_TESTS=true; shift ;;
        -h|--help) show_help; exit 0 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# Set colors if enabled
if [[ "$USE_COLOR" == "true" ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    NC='\033[0m'
fi

log_info() { echo -e "${YELLOW}$1${NC}"; }
log_success() { echo -e "${GREEN}✓ $1${NC}"; }
log_error() { echo -e "${RED}✗ $1${NC}"; }

run_local_unit_tests() {
    if [[ "$SKIP_UNIT_TESTS" == "true" ]]; then
        return 0
    fi
    
    log_info "Running local unit tests..."
    if cargo test --lib --bins; then
        log_success "Local unit tests passed"
    else
        log_error "Local unit tests failed"
        exit 1
    fi
}

build_docker_image() {
    log_info "Building Docker image (includes quality gates)..."
    if docker build -f "${DOCKERFILE}" -t "${IMAGE_NAME}:${TAG}" .; then
        log_success "Docker image built successfully"
    else
        log_error "Docker build failed"
        exit 1
    fi
}

run_integration_tests() {
    if [[ "$RUN_INTEGRATION_TESTS" == "false" ]]; then
        return 0
    fi

    log_info "Running integration tests against container..."
    
    local container_name="argus-test-$$"
    local cleanup_needed=false
    
    # Cleanup function
    cleanup_container() {
        if [[ "$cleanup_needed" == "true" ]]; then
            echo "Cleaning up container..."
            docker stop "$container_name" >/dev/null 2>&1 || true
            docker rm "$container_name" >/dev/null 2>&1 || true
        fi
    }
    
    # Set trap for cleanup
    trap cleanup_container EXIT
    
    # Start container
    if ! docker run -d --name "$container_name" -p 0:3000 "${IMAGE_NAME}:${TAG}" >/dev/null; then
        log_error "Failed to start container"
        exit 1
    fi
    cleanup_needed=true
    
    # Get mapped port and wait for startup
    local mapped_port
    mapped_port=$(docker port "$container_name" 3000/tcp | cut -d: -f2)
    echo "Container running on port: $mapped_port"
    echo "Waiting for container to start..."
    sleep 2
    
    # Run integration tests
    export ARGUS_TEST_BASE_URL="http://127.0.0.1:$mapped_port"
    if cargo test --test integration; then
        log_success "Integration tests passed"
    else
        log_error "Integration tests failed"
        echo -e "${YELLOW}Container logs:${NC}"
        docker logs "$container_name"
        exit 1
    fi
}

main() {
    log_info "Building Argus Events Docker image..."
    echo "Image: ${IMAGE_NAME}:${TAG}"
    echo "Dockerfile: ${DOCKERFILE}"
    echo
    
    run_local_unit_tests
    echo
    
    build_docker_image
    echo
    
    run_integration_tests
    echo
    
    log_info "Image details:"
    docker images "${IMAGE_NAME}:${TAG}"
    echo
    
    log_success "Build complete! 🚀"
    echo "To run the container:"
    echo "  docker run -p 3000:3000 ${IMAGE_NAME}:${TAG}"
    echo "Or use docker-compose:"
    echo "  docker-compose up"
}

main "$@"