//! Public gateway for the HTTP API layer.
//!
//! This module wires up Axum routes and exposes them for integration
//! into the main application.

mod events;

// Public exports (visible outside this module)
pub use events::event_routes;
