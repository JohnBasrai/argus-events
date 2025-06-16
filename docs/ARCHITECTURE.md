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
- **Explicit boundaries**: API, Service, and Repository layers are clearly defined
- **Data transformation**: Each boundary transforms data appropriately
- **Error boundaries**: Errors are handled at appropriate architectural layers

#### Principled (P)
- **SOLID principles**: Single Responsibility, Open/Closed, Liskov Substitution, Interface Segregation, Dependency Inversion
- **Domain-driven design**: Models reflect business domain concepts
- **Testability**: Architecture supports comprehensive unit and integration testing

## Layer Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         API Layer                            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   HTTP Routes   │  │   Validation    │  │   Metrics    │ │
│  │   (Axum)        │  │   (Serde)       │  │ (Prometheus) │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                       Service Layer                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ Business Logic  │  │   Aggregation   │  │ Validation   │ │
│  │                 │  │   Statistics    │  │ Rules        │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                     Repository Layer                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   Storage       │  │     Queries     │  │   Indexing   │ │
│  │   Abstraction   │  │   Filtering     │  │   Caching    │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Module Structure

### API Module (`src/api/`)
**Responsibility**: HTTP request/response handling, validation, and routing

- **Handlers**: Process HTTP requests and delegate to services
- **Middleware**: Cross-cutting concerns (logging, metrics, CORS)
- **Validation**: Input sanitization and validation
- **Serialization**: JSON request/response transformation

**Key Traits**:
```rust
trait EventHandler {
    async fn create_event(&self, request: CreateEventRequest) -> Result<EventResponse>;
    async fn get_events(&self, query: EventQuery) -> Result<Vec<EventResponse>>;
    async fn get_stats(&self) -> Result<StatsResponse>;
}
```

### Service Module (`src/service/`)
**Responsibility**: Business logic, domain rules, and orchestration

- **Event Service**: Core business logic for event processing
- **Stats Service**: Aggregation and statistical calculations
- **Validation**: Domain-specific validation rules
- **Orchestration**: Coordinate multiple repository operations

**Key Traits**:
```rust
trait EventService {
    async fn create_event(&self, event: Event) -> Result<Event>;
    async fn find_events(&self, criteria: EventCriteria) -> Result<Vec<Event>>;
    async fn calculate_stats(&self, timeframe: TimeRange) -> Result<EventStats>;
}
```

### Repository Module (`src/repository/`)
**Responsibility**: Data access, storage, and querying

- **Storage Abstraction**: Trait-based storage interface
- **Query Engine**: Filtering and searching capabilities
- **Indexing**: Efficient data retrieval strategies
- **Persistence**: Data storage and retrieval implementation

**Key Traits**:
```rust
trait EventRepository {
    async fn save(&self, event: Event) -> Result<Event>;
    async fn find_by_criteria(&self, criteria: EventCriteria) -> Result<Vec<Event>>;
    async fn count_by_type(&self, event_type: &str) -> Result<u64>;
    async fn find_unique_users(&self) -> Result<u64>;
}
```

### Models Module (`src/models/`)
**Responsibility**: Domain types, value objects, and data structures

- **Domain Models**: Core business entities (Event, User, Session)
- **Value Objects**: Immutable data containers
- **DTOs**: Data transfer objects for API boundaries
- **Enums**: Type-safe constants and classifications

### Error Module (`src/error/`)
**Responsibility**: Error types, error handling, and error transformation

- **Domain Errors**: Business logic errors
- **Infrastructure Errors**: Storage and external service errors
- **API Errors**: HTTP-specific error responses
- **Error Mapping**: Transform between error types at boundaries

## SOLID Principles Application

### Single Responsibility Principle (SRP)
- Each module has one reason to change
- API layer only handles HTTP concerns
- Service layer only handles business logic
- Repository layer only handles data access

### Open/Closed Principle (OCP)
- Trait-based interfaces allow extension without modification
- New storage backends can be added by implementing traits
- New event types can be added without changing existing code

### Liskov Substitution Principle (LSP)
- All implementations of storage traits are interchangeable
- Mock implementations can replace real implementations in tests
- Different storage backends (memory, database) behave consistently

### Interface Segregation Principle (ISP)
- Traits are focused and minimal
- Clients depend only on methods they use
- Separate read and write interfaces where appropriate

### Dependency Inversion Principle (DIP)
- High-level modules depend on abstractions (traits)
- Concrete implementations are injected at runtime
- Business logic is independent of infrastructure details

## Testing Strategy

### Unit Tests
- Individual functions and methods
- Mock dependencies using trait implementations
- Fast execution, high isolation

### Integration Tests
- Multiple components working together
- Real storage implementations
- API endpoint testing

### Property-Based Tests
- Generate random test data
- Verify invariants and properties
- Catch edge cases and corner cases

## Performance Considerations

### Async Performance
- Non-blocking I/O operations
- Efficient task scheduling with Tokio
- Connection pooling for external resources

### Memory Management
- Zero-copy operations where possible
- Efficient data structures (BTreeMap for sorted access)
- Memory-mapped files for large datasets

### Caching Strategy
- In-memory caching for frequently accessed data
- Cache invalidation strategies
- Lazy loading of expensive computations

## Extensibility Points

### Storage Backends
- In-memory (current implementation)
- PostgreSQL/MySQL database
- Redis for caching
- File-based storage

### Event Processing
- Real-time event streaming
- Batch processing capabilities
- Event sourcing and CQRS patterns

### Monitoring and Observability
- Prometheus metrics collection
- Distributed tracing with OpenTelemetry
- Structured logging with correlation IDs

### API Enhancements
- GraphQL interface
- WebSocket real-time updates
- API versioning strategies

---

This architecture provides a solid foundation for building scalable, maintainable event tracking systems while demonstrating advanced Rust patterns and design principles.