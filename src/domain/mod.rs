//! Public gateway for the domain module.
//!
//! This module exposes all core domain types and traits
//! used by the service layer and storage implementations.

// Bring all submodules into scope
mod event;
mod event_query;
mod metrics;
mod repository;

// Public exports (visible outside this module)
//b use crate::infrastructure::create_metrics;
pub use crate::repository::create_repository;
pub use event::Event;
pub use event_query::EventQuery;
pub use metrics::{Metrics, MetricsPtr};
pub use repository::{EventRepository, EventRepositoryPtr};
