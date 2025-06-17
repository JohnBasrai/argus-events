//! Crate-wide gateway for internal modules.
//!
//! This file re-exports only the public-facing symbols used by
//! the main binary or other consumers. All internal structure
//! is hidden behind this gateway.

// Bring submodules into scope
mod api;
mod cli;
mod domain;
mod infrastructure;
mod repository;

// Public exports (visible outside this crate)
pub use api::event_routes;
pub use cli::Args;
pub use domain::{
    // ------------
    create_repository,
    Event,
    EventQuery,
    EventRepository,
    EventRepositoryPtr,
    Metrics,
    MetricsPtr,
};
pub use infrastructure::create_metrics;

// Helper function for creating the complete app (useful for testing)
pub fn create_app(repo: EventRepositoryPtr, metrics: MetricsPtr) -> anyhow::Result<axum::Router> {
    // ---
    Ok(event_routes(repo, metrics))
}
