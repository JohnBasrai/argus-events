# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] – 2025-06-16

### Added
- Initial implementation of event tracking service using Axum and async Rust.
- `POST /events`: accepts events with `event_type`, `timestamp`, and free-form payload.
- `GET /events`: filters events by `type`, `start`, and `end`.
- In-memory storage backend using `DashMap`, with repository abstraction for future extensibility.
- Command-line interface using `clap` with support for `--endpoint` and `ARGUS_ENDPOINT`.
- Full test suite with 100% coverage for in-memory backend.
- Clean architecture following EMBP pattern (`domain`, `repository`, `api`, etc).
- Docs: `README.md`, `ARCHITECTURE.md`, `DEVELOPMENT.md`, `ASSIGNMENT.md`.

---

