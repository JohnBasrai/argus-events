# Technical Assessment - Event Tracking REST API

## Original Requirements

This document captures the original technical assessment requirements from CSide.

### Core Functionality
Build a REST API for event tracking with the following endpoints:

#### 1. POST /events
Create a new event with the following structure:
```json
{
  "event_type": "string",
  "timestamp": "ISO 8601 datetime",
  "user_id": "string",
  "session_id": "string", 
  "properties": {
    "key": "value pairs of additional event data"
  }
}
```

#### 2. GET /events
Retrieve events with optional query parameters:
- `user_id`: Filter by user ID
- `event_type`: Filter by event type
- `start_time`: Filter events after this timestamp
- `end_time`: Filter events before this timestamp
- `limit`: Maximum number of events to return (default: 100)
- `offset`: Number of events to skip for pagination

#### 3. GET /events/stats
Get aggregated statistics:
```json
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

### Technical Requirements

#### Must Have
- [x] Rust implementation using modern async patterns
- [x] Proper error handling with meaningful HTTP status codes
- [x] Input validation for all endpoints
- [x] JSON request/response format
- [x] In-memory storage (HashMap/Vec acceptable for assessment)
- [x] Comprehensive documentation
- [x] Unit tests for core functionality

#### Nice to Have (Stretch Goals)
- [ ] Prometheus metrics integration
- [ ] Docker containerization  
- [ ] Structured logging
- [ ] Database persistence layer
- [ ] API rate limiting
- [ ] OpenAPI/Swagger documentation

### Assessment Criteria
- **Code Quality**: Clean, readable, well-structured code
- **Architecture**: Demonstration of solid design principles
- **Testing**: Comprehensive test coverage
- **Documentation**: Clear setup and usage instructions
- **Error Handling**: Robust error management
- **Performance**: Efficient data handling and retrieval

### Delivery
- GitHub repository with complete source code
- README with setup and usage instructions
- Documentation explaining architectural decisions
- Runnable application with example usage

---

**Implementation Notes:**
- This assessment showcases EMBP (Event-driven, Modular, Boundary-aware, Principled) architecture
- Emphasizes SOLID principles and trait-based abstractions
- Uses mythological naming convention (Argus - the all-seeing giant, perfect for event tracking)
- Demonstrates production-ready Rust patterns and best practices