# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-06-16

### Added

#### 🎯 Core Infrastructure
- **Comprehensive metrics system** with Prometheus integration (`src/infrastructure/metrics/`)
  - Prometheus metrics backend with HTTP request duration histograms
  - NoOp metrics backend for testing/development
  - Environment-based metrics selection (`ARGUS_METRICS_TYPE=prom|noop`)
  - `/metrics` endpoint exposing Prometheus-formatted metrics

#### 🐳 Production-Ready Containerization
- **Multi-stage Dockerfile** using cr8s base images (`ghcr.io/johnbasrai/cr8s/rust-dev:1.83.0-rev5`)
- **Comprehensive quality gates** in Docker build:
  - Code formatting checks (`cargo fmt --check`)
  - Linting with warnings as errors (`cargo clippy`)
  - Security vulnerability scanning (`cargo audit`)
  - Unit test execution in release mode
- **Container-per-test integration testing** with automatic cleanup
- **Production runtime image** based on `cr8s/rust-runtime` (205MB)

#### 🔧 Build & Development Infrastructure  
- **Professional build script** (`scripts/build.sh`) with:
  - Color output support (auto-detected or forced)
  - Local unit testing with optional skip
  - Docker image building with quality gates
  - Container-based integration testing with port isolation
  - Automatic test container cleanup
- **GitHub Actions CI/CD pipeline** (`.github/workflows/ci.yml`):
  - Leverages existing build script for consistency
  - Uses cr8s dev image for fast, consistent builds
  - Security scanning with Trivy
  - Automated releases with container registry publishing

#### 📊 Advanced Testing Infrastructure
- **Container-per-test integration tests** (`tests/integration.rs`):
  - Each test gets isolated container instance
  - Environment-aware server selection (embedded vs containerized)
  - Ephemeral port allocation for concurrent test execution
  - Automatic container lifecycle management
- **Comprehensive metrics testing** (`tests/metrics_endpoint.rs`):
  - Prometheus metrics validation
  - Load testing with concurrent requests
  - Content-type verification
  - NoOp metrics fallback testing

#### 🚦 Production Features
- **Nginx reverse proxy** configuration with:
  - Rate limiting (10 requests/second with burst capacity)
  - Security headers (XSS protection, frame options)
  - Gzip compression for JSON responses
  - Health check endpoint proxying
- **Docker Compose** setup for development and production profiles
- **Structured logging** with contextual fields throughout request handlers

### Enhanced

#### 📝 API Layer Improvements
- **Enhanced HTTP handlers** (`src/api/events.rs`):
  - Comprehensive structured logging with event context
  - Real-time metrics recording for all endpoints
  - Detailed error handling with proper HTTP status codes
  - Request timing and performance tracking

#### 🏗️ Architecture Refinements
- **Application state management** with shared repository and metrics
- **Clean separation** between embedded testing and container testing
- **Environment-based configuration** for metrics backends
- **Improved error handling** with proper context propagation

### Changed
- **Function signatures** updated to support metrics integration:
  - `create_app()` now requires both repository and metrics parameters
  - `event_routes()` updated to use application state pattern
- **Integration tests** now return `Result<()>` for proper error propagation
- **Docker Compose** configuration updated to use pre-built images when available

### Dependencies
- **Added dev dependencies**:
  - `futures = "0.3.31"` for async utilities
  - `serial_test = "3.2"` for test synchronization
  - `reqwest = { version = "0.11", features = ["json"] }` for HTTP testing

### Infrastructure
- **Build script improvements** with trap-based container cleanup
- **CI pipeline** simplified to ~10 lines by leveraging existing build infrastructure
- **Container registry** integration for automated releases

### Documentation
- **Updated README.md** with comprehensive testing strategy documentation
- **Enhanced project structure** showing testing and build infrastructure
- **Container-per-test methodology** explanation for interview showcase

---

## [0.1.0] - 2025-06-16

### Added
- Initial implementation of Argus Events tracking service
- REST API with `/events` endpoint (GET/POST)
- In-memory event storage with concurrent-safe operations
- Event filtering by type and time range
- Comprehensive unit tests for core functionality
- Basic Docker support
- Command-line argument parsing with environment variable fallbacks
- Structured error handling with anyhow
- UUID-based event identification
- JSON request/response handling

### Core Features
- **Event submission** via POST `/events` with JSON payload
- **Event querying** via GET `/events` with optional filters:
  - `type`: Filter by event type
  - `start`/`end`: Time range filtering with RFC3339 timestamps
- **Thread-safe in-memory storage** using DashMap for concurrent access
- **Clean architecture** with trait-based repository abstraction

### Technical Implementation
- **Axum-based REST API** with async/await throughout
- **Trait-based storage abstraction** for future extensibility
- **Comprehensive error handling** with proper HTTP status codes
- **Input validation** for timestamps and required fields
- **UUID generation** for unique event identification

[Unreleased]: https://github.com/johnbasrai/argus-events/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/johnbasrai/argus-events/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/johnbasrai/argus-events/releases/tag/v0.1.0
