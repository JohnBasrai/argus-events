# Argus Events
High-performance event tracking service in Rust showcasing clean architecture patterns
 
> Named after Argus Panoptes, the all-seeing giant of Greek mythology who never slept - much like this service monitors and tracks events across your system with vigilant oversight.

A high-performance event tracking service built with Rust, demonstrating clean architecture principles and production-ready patterns for handling concurrent event ingestion and querying.

## Features

- **REST API** for event submission and querying
- **Concurrent-safe in-memory storage** with future database migration support
- **Trait-based architecture** enabling pluggable storage backends
- **Comprehensive error handling** with structured error types
- **Production metrics** with Prometheus integration
- **Docker support** for easy deployment
- **Extensive testing** including integration tests

## Architecture

Argus Events showcases the **Explicit Module Boundary Pattern (EMBP)** and **SOLID principles** through a clean layered architecture:

```
src/
├── lib.rs              ← Main gateway
├── api/                ← HTTP layer (Axum-based REST API)
├── service/            ← Business logic layer
├── repository/         ← Data access abstraction
├── models/             ← Domain types
└── error/              ← Structured error handling
tests/
├── integration.rs      ← integration tests
└── metrics_endpoint.rs ← metrics collection test
scripts/
└── build.sh            ← Quality pipeline build script
```

### Key Design Principles

- **Dependency Inversion**: Service layer depends on storage traits, not concrete implementations
- **Single Responsibility**: Clear separation between HTTP, business logic, and data layers
- **Interface Segregation**: Focused traits for different storage operations
- **EMBP**: Module boundaries enforced through explicit gateways (`mod.rs` files)

## Quick Start

### Prerequisites

- Rust 1.75+ 
- Docker (optional)

### Running Locally

```bash
# Clone the repository
git clone https://github.com/YourUsername/argus-events.git
cd argus-events

# Run the service
cargo run

# The API will be available at http://localhost:3000
```

### Using Docker

```bash
# Build and run with Docker Compose
docker-compose up --build

# Or build and run manually
docker build -t argus-events .
docker run -p 3000:3000 argus-events
```

## API Usage

```
# Run the API server (env fallback or CLI arg required)
cargo run -- --endpoint 127.0.0.1:3000
```

### Submit Events

```bash
POST /events
Content-Type: application/json

{
  "event_type": "user_signup",
  "timestamp": 1640995200,
  "payload": {
    "user_id": "12345",
    "email": "user@example.com",
    "source": "web"
  }
}
```

### Query Events

```bash
# Get all events
GET /events

# Filter by event type
GET /events?type=user_signup

# Filter by time range
GET /events?start=1640995200&end=1640998800

# Combine filters
GET /events?type=user_signup&start=1640995200&end=1640998800
```

## Development

### Project Structure

The codebase follows EMBP (Explicit Module Boundary Pattern) for clean architectural boundaries:

- **API Layer**: HTTP request handling, serialization, validation
- **Service Layer**: Core business logic, event processing
- **Repository Layer**: Data access abstraction through traits
- **Models**: Domain types and data structures
- **Error Handling**: Structured error types with proper context

### Storage Architecture

The storage layer uses trait-based abstraction for future extensibility:

```rust
pub trait EventRepository: Send + Sync {
    async fn store_event(&self, event: Event) -> Result<(), RepositoryError>;
    async fn find_events(&self, query: EventQuery) -> Result<Vec<Event>, RepositoryError>;
}
```

Current implementation uses concurrent-safe in-memory storage, designed for easy migration to persistent backends like PostgreSQL.

## Testing Strategy

Argus Events demonstrates production-quality testing with a comprehensive multi-tier approach:

### Container-Per-Test Integration Testing

The integration tests showcase advanced Docker-based testing where **each test gets its own isolated container**:

```bash
# Run the full test suite including container isolation
./scripts/build.sh --color

# This will:
# 1. Run local unit tests
# 2. Build Docker image with quality gates (lint, format, security audit)
# 3. Run integration tests with isolated containers per test
# 4. Automatically clean up all test containers
```

**Key Features:**
- **True isolation**: Each test starts a fresh container with clean state
- **Production parity**: Tests run against the actual Docker image that gets deployed  
- **Concurrent safe**: Tests can run in parallel without interference
- **Automatic cleanup**: Test containers are removed automatically
- **Environment detection**: Smart switching between embedded servers (local dev) and containers (CI)

### Testing Layers

```bash
# Unit tests (fast, isolated business logic)
cargo test --lib

# Integration tests (container-based, production-like)
cargo test --test integration

# Full quality pipeline
./scripts/build.sh
```

The testing approach demonstrates:
- **Docker expertise** with dynamic container lifecycle management
- **Resource management** with proper cleanup and error handling
- **Test isolation principles** ensuring reliable, repeatable tests
- **CI/CD readiness** with environment-aware test strategies

### Metrics

Prometheus metrics are available at `/metrics`:

- Request counts and response times
- Event ingestion rates
- Error rates by type
- Memory usage statistics

## Production Considerations

This project demonstrates production-ready patterns:

- **Graceful shutdown** handling
- **Request timeout** management
- **Structured logging** with configurable levels
- **Health check** endpoint (`/health`)
- **Metrics collection** for observability
- **Container-ready** with multi-stage Docker builds

## Technical Highlights

### Concurrency & Safety

- **Async/await** throughout using Tokio runtime
- **Thread-safe storage** with appropriate synchronization primitives
- **Concurrent request handling** without blocking operations

### Error Handling

- **Custom error types** with context preservation
- **Graceful degradation** for non-critical failures
- **Proper HTTP status codes** for different error scenarios

### Testing Strategy

- **Unit tests** for business logic
- **Integration tests** for API endpoints
- **Property-based testing** for data validation
- **Mock implementations** for trait testing

## Future Enhancements

- [ ] **Rate limiting** per IP address
- [ ] **Event streaming** with WebSocket support
- [ ] **Persistent storage** backends (PostgreSQL, MongoDB)
- [ ] **Event aggregation** and analytics
- [ ] **Authentication** and authorization
- [ ] **Horizontal scaling** with clustering support

## Contributing

This project serves as both a functional event tracking service and a demonstration of Rust architectural patterns. While primarily for educational purposes, contributions and feedback are welcome!

### Notes

- Graceful shutdown is now implemented using `tokio::signal::ctrl_c()` and Axum’s `with_graceful_shutdown`.
  Although initially marked complete in `ASSIGNMENT.md`, this was properly wired up and manually verified later.

### Requirement Coverage

For a complete mapping of all assignment requirements to source code (with line numbers), see:

➡️ [`docs/ASSIGNMENT.md`](docs/ASSIGNMENT.md) → “✅ Requirement Verification Matrix (from tag: v0.2.3)”

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## About the Name

Argus Panoptes was a giant in Greek mythology covered with eyes, making him an excellent watchman who could see in all directions and never fully slept. Just as Argus kept vigilant watch over everything in his domain, this service maintains continuous oversight of events flowing through your system, ensuring nothing escapes notice.
