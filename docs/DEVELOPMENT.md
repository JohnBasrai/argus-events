# Development Guide

## Quick Start

### Prerequisites
- Rust 1.70+ (2021 edition)
- Git
- Docker (optional, for containerized deployment)

### Setup
```bash
# Clone the repository
git clone https://github.com/yourusername/argus-events.git
cd argus-events

# Build the project
cargo build

# Run tests
cargo test

# Run the API server (env fallback or CLI arg required)
cargo run -- --endpoint 127.0.0.1:3000
```

The API will be available at `http://localhost:3000`

## Development Environment

### Recommended Tools
- **IDE**: VS Code with rust-analyzer extension
- **HTTP Client**: curl, HTTPie, or Postman
- **Monitoring**: Prometheus (for metrics collection)

### Environment Variables
```bash
# Server configuration
export ARGUS_HOST=127.0.0.1
export ARGUS_PORT=3000

# Logging
export RUST_LOG=debug

# Metrics (optional)
export METRICS_ENABLED=true
export METRICS_PORT=9090
```

## Testing Strategy

### Running Tests
```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --test integration

# Run with coverage
cargo tarpaulin --out Html
```

### Test Categories

#### Unit Tests
Located alongside source code in `src/` modules:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_event() {
        // Test implementation
    }
}
```

#### Integration Tests
Located in `tests/` directory:
```rust
// tests/api_integration.rs
#[tokio::test]
async fn test_full_event_lifecycle() {
    // End-to-end API testing
}
```

#### Property-Based Tests
Using the `proptest` crate for random testing:
```rust
proptest! {
    #[test]
    fn test_event_serialization(event in event_strategy()) {
        // Property testing
    }
}
```

### Mock Testing
The architecture supports comprehensive mocking through traits:

```rust
// Mock repository for testing
pub struct MockEventRepository {
    events: Arc<Mutex<Vec<Event>>>,
}

#[async_trait]
impl EventRepository for MockEventRepository {
    async fn save(&self, event: Event) -> Result<Event> {
        // Mock implementation
    }
}
```

## Code Quality

### Linting and Formatting
```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy -- -D warnings

# Check for security vulnerabilities
cargo audit
```

### Code Coverage
Target: 90%+ test coverage for critical paths

```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/

# View coverage
open coverage/tarpaulin-report.html
```

## API Usage Examples

### Creating Events
```bash
# Create a page view event
curl -X POST http://localhost:3000/events \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "page_view",
    "timestamp": "2025-06-16T12:00:00Z",
    "user_id": "user_123",
    "session_id": "session_456",
    "properties": {
      "page": "/dashboard",
      "referrer": "https://google.com"
    }
  }'
```

### Querying Events
```bash
# Get all events for a user
curl "http://localhost:3000/events?user_id=user_123"

# Get events by type with time range
curl "http://localhost:3000/events?event_type=page_view&start_time=2025-06-16T00:00:00Z&limit=50"

# Get paginated results
curl "http://localhost:3000/events?limit=10&offset=20"
```

### Getting Statistics
```bash
# Get overall statistics
curl http://localhost:3000/events/stats

# Example response:
{
  "total_events": 1000,
  "unique_users": 50,
  "events_by_type": {
    "page_view": 400,
    "click": 300,
    "purchase": 300
  },
  "events_last_24h": 150
}
```

## Performance Testing

### Load Testing
Using `wrk` for HTTP load testing:

```bash
# Install wrk
brew install wrk  # macOS
sudo apt-get install wrk  # Ubuntu

# Load test event creation
wrk -t12 -c400 -d30s -s scripts/create_event.lua http://localhost:3000/events

# Load test event querying
wrk -t8 -c100 -d30s http://localhost:3000/events
```

### Benchmarking
```bash
# Run cargo benchmarks
cargo bench

# Profile with flamegraph
cargo flamegraph --bin argus-events
```

## Monitoring and Observability

### Prometheus Metrics
When enabled, metrics are available at `http://localhost:9090/metrics`

Key metrics:
- `argus_events_total`: Total events processed
- `argus_requests_duration_seconds`: Request latency histogram
- `argus_active_connections`: Current active connections

### Logging
Structured logging with correlation IDs:
```rust
info!(
    user_id = %event.user_id,
    event_type = %event.event_type,
    correlation_id = %correlation_id,
    "Event created successfully"
);
```

### Health Checks
```bash
# Basic health check
curl http://localhost:3000/health

# Detailed health check
curl http://localhost:3000/health/detailed
```

## Docker Deployment

### Building the Image
```bash
# Build production image
docker build -t argus-events:latest .

# Run container
docker run -p 3000:3000 argus-events:latest
```

### Docker Compose
```yaml
# docker-compose.yml
version: '3.8'
services:
  argus-events:
    build: .
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## Contribution Guidelines

### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Use descriptive variable and function names
- Add documentation for public APIs
- Include examples in documentation

### Commit Messages
```
feat: add event aggregation endpoint
fix: resolve memory leak in event storage
docs: update API documentation
test: add integration tests for stats endpoint
refactor: extract validation logic to separate module
```

### Pull Request Process
1. Create feature branch from `main`
2. Implement changes with tests
3. Ensure all tests pass
4. Update documentation
5. Submit PR with clear description

### Code Review Checklist
- [ ] Tests pass and coverage is maintained
- [ ] Code follows style guidelines
- [ ] Documentation is updated
- [ ] No security vulnerabilities
- [ ] Performance impact is acceptable
- [ ] Error handling is appropriate

## Troubleshooting

### Common Issues

#### Port Already in Use
```bash
# Find process using port 3000
lsof -i :3000

# Kill the process
kill -9 <PID>
```

#### Memory Issues
```bash
# Monitor memory usage
htop

# Profile memory usage
valgrind --tool=massif target/debug/argus-events
```

#### Slow Tests
```bash
# Run tests with timing
cargo test -- --nocapture --test-threads 1 -Z unstable-options --report-time
```

## Advanced Features

### Custom Storage Backends
Implement the `EventRepository` trait:

```rust
pub struct PostgresEventRepository {
    pool: PgPool,
}

#[async_trait]
impl EventRepository for PostgresEventRepository {
    async fn save(&self, event: Event) -> Result<Event> {
        // PostgreSQL implementation
    }
}
```

### Event Streaming
Add real-time event streaming:

```rust
pub struct EventStream {
    tx: broadcast::Sender<Event>,
}

impl EventStream {
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }
}
```

### Rate Limiting
Add rate limiting middleware:

```rust
pub async fn rate_limit_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response<Body>, StatusCode> {
    // Rate limiting implementation
}
```

---

This development guide provides comprehensive information for working with the Argus Events API, from basic setup to advanced customization options.
