#!/bin/bash

# Enhanced build script with better error detection

set -euo pipefail

# Default values
IMAGE_NAME="argus-events"
TAG="latest"
DOCKERFILE="Dockerfile"
VERBOSE_TEST=""
RUN_INTEGRATION_TESTS=true
USE_COLOR=true

# Colors (empty by default)
RED=''
GREEN=''
YELLOW=''
NC=''

# Auto-detect colors: TTY + not CI + not NO_COLOR
if [[ -t 1 && "${NO_COLOR:-}" != "1" && "${CI:-}" != "true" ]]; then
    USE_COLOR=false
fi

show_help() {
    cat << EOF
Usage: $0 [OPTIONS]
Options:
  -t, --tag              Set image tag (default: latest)
  -n, --name             Set image name (default: argus-events)
  -f, --file             Set Dockerfile path (default: Dockerfile)
  --verbose              Generate verbose unit/integration test output
  --color                Force colored output
  --no-color             Disable colored output (default)
  --skip-integration     Skip integration tests against container
  -h, --help             Show this help message

Colors are auto-detected based on terminal support.
You can also set NO_COLOR=1 environment variable to disable colors.
Command line options override environment variables.
EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--tag) TAG="$2"; shift 2 ;;
        -n|--name) IMAGE_NAME="$2"; shift 2 ;;
        -f|--file) DOCKERFILE="$2"; shift 2 ;;
        --verbose) VERBOSE_TEST="--nocapture"; shift;;
        --color) USE_COLOR=true; shift ;;
        --no-color) USE_COLOR=false; shift ;;
        --skip-integration) RUN_INTEGRATION_TESTS=false; shift ;;
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

build_docker_image() {

    log_info "Building Docker image (includes quality gates)..."
    
    # Capture build output to check for warnings/errors
    local build_log=$(mktemp)
    local build_success=true
    
    if docker build -f "${DOCKERFILE}" -t "${IMAGE_NAME}:${TAG}" . 2>&1 | tee "$build_log"; then
        # Check for security audit failures in build log
        if grep -v "^#[0-9].*\[.*\] RUN" "$build_log" | \
                grep -q "❌ Security audit failed - check for vulnerabilities"; then
            log_error "Security vulnerabilities detected during build!"
            log_error "Review the audit output above and address security issues."
            build_success=false
        fi
        
        # Check for other concerning patterns (but exclude expected warnings)
        if grep -v "^#[0-9].*\[.*\] RUN" "$build_log" | \
                grep -q "error:\|Error:\|failed to\|Failed to" | grep -v "FromAsCasing"; then
            log_error "Build completed but errors were detected in output"
            build_success=false
        fi
        
        # Check for warnings (informational only)
        if grep -v "^#[0-9].*\[.*\] RUN" "$build_log" | grep -q "warning:\|Warning:"; then
            log_info "⚠️  Build warnings detected - review output above"
        fi
        
        rm -f "$build_log"
        
        if [[ "$build_success" == "true" ]]; then
            log_success "Docker image built successfully"
        else
            log_error "Build completed but with critical issues"
            exit 1
        fi
    else
        rm -f "$build_log"
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
    trap "cleanup_needed=true container_name=$container_name; cleanup_container" EXIT
    
    # Start container
    if ! docker run -d --name "$container_name" -p 0:3000 "${IMAGE_NAME}:${TAG}" >/dev/null; then
        log_error "Failed to start container"
        exit 1
    fi
    cleanup_needed=true
    
    # Get mapped port and wait for startup with timeout
    local mapped_port
    mapped_port=$(docker port "$container_name" 3000/tcp | cut -d: -f2)
    echo "Container running on port: $mapped_port"
    echo "Waiting for container to start..."
    
    # Wait for container to be ready (basic approach)
    sleep 3
    
    # Run integration tests with output capture
    export ARGUS_TEST_BASE_URL="http://127.0.0.1:$mapped_port"
    local test_log=$(mktemp)
    local test_success=true
    
    if cargo test -- ${VERBOSE_TEST} 2>&1 | tee "$test_log"; then
        # Check for test failures or concerning output
        if grep -q "FAILED\|panicked at\|thread.*panicked" "$test_log"; then
            log_error "Tests reported as passed but failures detected in output"
            test_success=false
        fi
        
        # Show test summary
        local test_count=$(grep -c "test.*ok" "$test_log" 2>/dev/null || echo "0")
        if [[ "$test_success" == "true" ]]; then
            log_success "Integration tests passed ($test_count tests)"
        fi
    else
        log_error "Integration tests failed"
        echo -e "${YELLOW}Container logs:${NC}"
        docker logs "$container_name"
        test_success=false
    fi
    
    rm -f "$test_log"
    
    if [[ "$test_success" == "false" ]]; then
        exit 1
    fi
}

validate_build() {

    log_info "Validating final image..."
    
    # Check image size (informational)
    local image_size=$(docker images "${IMAGE_NAME}:${TAG}" --format "{{.Size}}" 2>/dev/null || echo "unknown")
    echo "Final image size: $image_size"
    
    # Basic validation - check that image exists
    if docker images "${IMAGE_NAME}:${TAG}" --format "{{.Repository}}" | grep -q "${IMAGE_NAME}"; then
        log_success "Image validation passed"
    else
        log_error "Image validation failed - image not found"
        exit 1
    fi
}

main() {

    log_info "Building Argus Events Docker image..."
    echo "Image: ${IMAGE_NAME}:${TAG}"
    echo "Dockerfile: ${DOCKERFILE}"
    echo
    
    build_docker_image
    echo
    
    run_integration_tests
    echo
    
    validate_build
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
