//! Repository trait for storing and querying events.
//!
//! This trait defines the abstract interface for event storage,
//! allowing the system to remain decoupled from any specific backend.
//! Implementations may use in-memory data structures, databases, or
//! external services.

#![allow(dead_code)]

use async_trait::async_trait;

use super::{Event, EventQuery};

/// Trait representing a pluggable event storage backend.
#[async_trait]
pub trait EventRepository: Send + Sync {
    /// Stores a new event in the underlying backend.
    async fn store_event(&self, event: Event) -> anyhow::Result<()>;

    /// Retrieves events matching the given query filters.
    async fn find_events(&self, query: EventQuery) -> anyhow::Result<Vec<Event>>;
}

/// Shared, thread-safe pointer to a dynamic EventRepository implementation.
pub type EventRepositoryPtr = std::sync::Arc<dyn EventRepository>;
