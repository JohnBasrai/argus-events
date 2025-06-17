# Architecture Documentation

## EMBP Pattern Implementation

The Argus Events API is built using the **EMBP** (Event-driven, Modular, Boundary-aware, Principled) architecture pattern, which emphasizes clean separation of concerns and maintainable code structure.

### Core Principles

#### Event-driven (E)
- **Async-first design**: All I/O operations use Tokio async patterns
- **Event sourcing ready**: Architecture supports future event sourcing implementations
- **Reactive patterns**: Components react to events rather than imperative calls

#### Modular (M)
- **Clear module boundaries**: Each module has a single responsibility
- **Trait-based abstractions**: Interfaces define contracts, not implementations
- **Dependency injection**: Components depend on traits, not concrete types

#### Boundary-aware (B)
- **Explicit boundaries**: API, Domain, and Repository layers are clearly defined
- **Data transformation**: Each boundary transforms data appropriately
- **Error boundaries**: Errors are handled at appropriate architectural layers

#### Principled (P)
- **SOLID principles**: Single Responsibility, Open/Closed, Liskov Substitution, Interface Segregation, Dependency Inversion
- **Domain-driven design**: Models reflect business domain concepts
- **Testability**: Architecture supports comprehensive unit and integration testing

## Pragmatic 3-Layer Architecture

The current implementation follows a **pragmatic clean architecture** appropriate for the problem domain:

```
┌─────────────────────────────────────────────────────────────┐
│                         API Layer                            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   HTTP Routes   │  │   JSON I/O      │  │   Metrics    │ │
│  │   (Axum)        │  │   (Serde)       │  │ Endpoints    │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │
                    Direct delegation to domain
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                       Domain Layer                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   Core Models   │  │  Query Objects  │  │   Metrics    │ │
│  │   (Event)       │  │ (EventQuery)    │  │  Traits      │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                   Infrastructure Layer                       │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   Repository    │  │     Metrics     │  │   Config     │ │
│  │ Implementations │  │ (Prometheus)    │  │  (Factory)   │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
│  Memory • NoOp      │  Prometheus • NoOp  │  CLI • Env     │ │
└─────────────────────────────────────────────────────────────┘
```

**Why No Service Layer?**
- Simple CRUD operations don't require complex business orchestration
- API handlers are thin and focused (parse → validate → delegate → respond)
- Domain logic is minimal (basic event storage and querying)
- Adding unnecessary abstraction would violate YAGNI principle
- Current design follows the assignment requirements precisely

## Module Structure

### API Module (`src/api/`)
**Responsibility**: HTTP request/response handling, routing, and API contracts

- **Event Routes**: RESTful endpoints for event operations (`POST /events`, `GET /events`)
- **Metrics Endpoint**: Prometheus metrics exposure (`GET /metrics`)
- **Request Validation**: Input sanitization and validation using Serde
- **Response Formatting**: JSON serialization and HTTP status code handling
- **State Management**: Axum state containing repository and metrics dependencies

**Key Components**:
```rust
// HTTP handlers
async fn submit_event(State(state): State<AppState>, Json(input): Json<EventInput>) -> impl IntoResponse
async fn get_events(State(state): State<AppState>, Query(params): Query<GetEventsQuery>) -> impl IntoResponse
async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse

// Router factory
pub fn event_routes(repo: EventRepositoryPtr, metrics: MetricsPtr) -> Router
```

### Domain Module (`src/domain/`)
**Responsibility**: Core business models, traits, and value objects

- **Event Model**: Core domain entity representing tracked events
- **Event Query**: Value object for filtering and searching events
- **Repository Trait**: Abstract interface for event storage
- **Metrics Trait**: Abstract interface for metrics collection
- **Type Aliases**: Shared pointer types for dependency injection

**Key Types**:
```rust
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
}

pub struct EventQuery {
    pub event_type: Option<String>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

pub trait EventRepository: Send + Sync {
    async fn store_event(&self, event: Event) -> anyhow::Result<()>;
    async fn find_events(&self, query: EventQuery) -> anyhow::Result<Vec<Event>>;
}

pub trait Metrics: Send + Sync + 'static {
    fn render(&self) -> String;
    fn record_event_created(&self);
    fn record_http_request(&self, start: Instant, path: &str, method: &str, status: u16);
}
```

### Repository Module (`src/repository/`)
**Responsibility**: Concrete implementations of the `EventRepository` trait from domain

- **Memory Repository**: Thread-safe in-memory storage using DashMap
- **Noop Repository**: No-op implementation for testing/development
- **Factory Function**: Runtime selection of repository implementation
- **Query Processing**: Filtering by event type and time ranges

**Trait Implementations**:
```rust
// Both implement the domain trait
impl EventRepository for InMemoryEventRepository {
    async fn store_event(&self, event: Event) -> anyhow::Result<()> { ... }
    async fn find_events(&self, query: EventQuery) -> anyhow::Result<Vec<Event>> { ... }
}

impl EventRepository for NoopRepository {
    async fn store_event(&self, _event: Event) -> anyhow::Result<()> { ... }
    async fn find_events(&self, _query: EventQuery) -> anyhow::Result<Vec<Event>> { ... }
}

// Factory function returns trait objects
pub fn create_repository(kind: &str) -> Result<EventRepositoryPtr>
```

### Infrastructure Module (`src/infrastructure/`)
**Responsibility**: Concrete implementations of the `Metrics` trait and external system integration

- **Prometheus Metrics**: Production metrics using metrics-exporter-prometheus
- **Noop Metrics**: No-op implementation for testing/development  
- **Factory Functions**: Environment-based component selection
- **Configuration**: Runtime selection via environment variables

**Trait Implementations**:
```rust
// Both implement the domain trait
impl Metrics for PrometheusMetrics {
    fn render(&self) -> String { ... }
    fn record_event_created(&self) { ... }
    fn record_http_request(&self, start: Instant, path: &str, method: &str, status: u16) { ... }
}

impl Metrics for NoopMetrics {
    fn render(&self) -> String { String::new() }
    fn record_event_created(&self) {}
    fn record_http_request(&self, _: Instant, _: &str, _: &str, _: u16) {}
}

// Factory function returns trait objects
pub fn create_metrics() -> Result<MetricsPtr>
```

## Dependency Inversion in Action

The architecture demonstrates clear dependency inversion where:

1. **Domain defines contracts** via traits (`EventRepository`, `Metrics`)
2. **Infrastructure implements contracts** with concrete types
3. **API depends on abstractions** via trait objects (`EventRepositoryPtr`, `MetricsPtr`)
4. **Runtime injection** happens via factory functions

```rust
// Domain layer defines the contract
pub trait EventRepository: Send + Sync {
    async fn store_event(&self, event: Event) -> anyhow::Result<()>;
    async fn find_events(&self, query: EventQuery) -> anyhow::Result<Vec<Event>>;
}

// Infrastructure implements the contract
impl EventRepository for InMemoryEventRepository { ... }
impl EventRepository for NoopRepository { ... }

// API depends on the abstraction
#[derive(Clone)]
pub struct AppState {
    pub repo: EventRepositoryPtr,    // Arc<dyn EventRepository>
    pub metrics: MetricsPtr,         // Arc<dyn Metrics>
}

// Runtime assembly via factories
let repo = create_repository(&args.repository)?;     // Returns trait object
let metrics = create_metrics()?;                     // Returns trait object
let app = event_routes(repo, metrics);               // Injects dependencies
```

## SOLID Principles Application

### Single Responsibility Principle (SRP)
- **API Module**: Only handles HTTP concerns and routing
- **Domain Module**: Only contains core business models and interfaces
- **Repository Module**: Only handles data storage and retrieval
- **Infrastructure Module**: Only handles external system integration

### Open/Closed Principle (OCP)
- New repository backends can be added by implementing `EventRepository` trait
- New metrics backends can be added by implementing `Metrics` trait
- Factory functions allow extension without modifying existing code

### Liskov Substitution Principle (LSP)
- All `EventRepository` implementations are interchangeable
- All `Metrics` implementations are interchangeable
- Mock implementations can replace real implementations in tests

### Interface Segregation Principle (ISP)
- `EventRepository` trait is focused only on event storage operations
- `Metrics` trait is focused only on metrics collection
- No client depends on methods it doesn't use

### Dependency Inversion Principle (DIP)
- API layer depends on domain traits, not concrete implementations
- Repository implementations are injected via factory functions
- Business logic is independent of infrastructure details

## Testing Strategy

### Unit Tests
- **Repository Tests**: In-memory storage behavior, filtering logic
- **Domain Tests**: Model validation and query parameter parsing
- **Isolated Components**: Each module tested independently

### Integration Tests
- **End-to-End API**: Full HTTP request/response cycles
- **Container Testing**: Docker-based testing with real services
- **Concurrent Access**: Multi-threaded repository operations

### Test Infrastructure
- **Test Servers**: Ephemeral port binding for parallel test execution
- **Container Management**: Automatic cleanup and port management
- **Environment Switching**: Container vs. local test modes

## Current Implementation Status

### Implemented Features
✅ **Event Storage**: In-memory repository with concurrent access  
✅ **Event Querying**: Filtering by type and time range  
✅ **HTTP API**: RESTful endpoints for events and metrics  
✅ **Metrics Collection**: Prometheus integration with HTTP metrics  
✅ **Testing Infrastructure**: Comprehensive unit and integration tests  
✅ **Configuration**: Environment-based component selection  
✅ **CLI Support**: Command-line argument parsing  

### Architecture Patterns Used
- **Dependency Injection**: Trait-based abstractions with factory functions
- **Repository Pattern**: Abstract storage interface with multiple implementations
- **Factory Pattern**: Runtime component selection based on configuration
- **Strategy Pattern**: Pluggable metrics and storage backends

## Performance Considerations

### Async Performance
- All I/O operations use Tokio async patterns
- Non-blocking request handling with Axum
- Efficient task scheduling and resource utilization

### Memory Management
- DashMap for efficient concurrent access patterns
- Zero-copy operations where possible
- Efficient JSON serialization with Serde

### Concurrent Access
- Thread-safe repository implementations
- Lock-free data structures (DashMap)
- Shared state via Arc<> smart pointers

## Future Extensibility Points

### When a Service Layer Would Be Added
A service layer would become valuable when implementing:
- **Complex Business Rules**: Multi-step validation, business workflows
- **Event Aggregation**: Real-time statistics, complex analytics
- **Multi-Repository Operations**: Transactions, data consistency
- **External Integrations**: Webhooks, notifications, third-party APIs
- **Background Processing**: Async event processing, batch operations

### Future Storage Backends
- **PostgreSQL**: Persistent database storage
- **Redis**: High-performance caching layer
- **File System**: Simple file-based persistence
- **Event Sourcing**: Append-only event store

### Future API Enhancements
- **GraphQL**: Query language interface
- **WebSocket**: Real-time event streaming
- **Batch Operations**: Bulk event submission
- **API Versioning**: Multiple API versions
- **Statistics Endpoint**: Aggregated event analytics

### Future Monitoring
- **Distributed Tracing**: OpenTelemetry integration
- **Health Checks**: Service health endpoints
- **Structured Logging**: Correlation IDs and context

## Development Workflow

### Local Development
- In-memory storage for fast iteration
- No-op metrics for reduced complexity
- Comprehensive unit test coverage

### Testing
- Container-based integration tests
- Parallel test execution with ephemeral ports
- Both local and containerized test modes

### Production Deployment
- Prometheus metrics for monitoring
- Configurable storage backends
- Environment-based configuration

---

This architecture provides a clean, maintainable foundation for event tracking while demonstrating modern Rust patterns and clean architecture principles. The current implementation focuses on essential functionality with clear extension points for future enhancements.