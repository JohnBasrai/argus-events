# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## \[0.2.1] – 2025-06-17

### Added

* Unit tests for all remaining assignment requirements, including:

  * `GET /events?type=xyz&start=...&end=...` filtering combinations
  * Validation of time ranges (`start < end`)
  * Edge cases: empty filters, invalid timestamps, URL encoding

### Fixed

* Production bugs discovered via test-driven development:

  * Missing time range validation (`start >= end`)
  * Improper error handling in `/metrics` endpoint

### Changed

* Improved `/metrics` endpoint with Prometheus rendering fallback and structured error reporting.
* Enhanced `build.sh`:

  * Color output, verbose test mode, audit fail-fast, and robust cleanup traps.
* Refactored and compressed the `v0.2.0` changelog for clarity.
* Hardened integration test helpers and resolved lifetime issues.

---

## \[0.2.0] – 2025-06-16

### Added

* **Metrics system**: Prometheus-backed metrics with `/metrics` endpoint, NoOp fallback, and environment-based selection.
* **Containerized testing**: One-container-per-test model with isolated ports and automatic cleanup.
* **CI/CD integration**: Unified build script, GitHub Actions pipeline, Trivy security scanning, and automated container publishing.
* **Production-ready Docker setup**: Multi-stage builds using cr8s base images and hardened runtime image (205MB).
* **Nginx proxy**: Rate limiting, security headers, Gzip, and health check routing.

### Changed

* Refactored app state and route handlers for shared metrics/repo access.
* Enhanced error handling, logging, and test coverage across API and metrics.
* Docker Compose updated to support dev and prod profiles.

### Dependencies

* Added: `futures`, `serial_test`, `reqwest` (for HTTP + test orchestration).

### Documentation

* Updated README and structure diagrams to reflect new architecture and test strategy.

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

[Unreleased]: https://github.com/johnbasrai/argus-events/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/johnbasrai/argus-events/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/johnbasrai/argus-events/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/johnbasrai/argus-events/releases/tag/v0.1.0
