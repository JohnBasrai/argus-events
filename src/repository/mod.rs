//! Repository implementations and factory function.
//!
//! This module provides concrete implementations of the EventRepository trait
//! and a factory function to create repository instances based on configuration.

mod memory;
mod noop_repository;

// Public exports
pub use crate::domain::EventRepositoryPtr;
use anyhow::Result;
use memory::create as create_memory_repository;
use noop_repository::create as create_noop_repository;

/// Factory function to create repository instances based on type string
pub fn create_repository(kind: &str) -> Result<EventRepositoryPtr> {
    // ---
    match kind {
        "memory" => create_memory_repository(),
        "noop" => create_noop_repository(),
        other => Err(anyhow::anyhow!("Unknown repository type: '{}'", other)),
    }
}
