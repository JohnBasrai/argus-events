//! Query parameters used to filter events during retrieval.
//!
//! This struct supports optional filtering by event type and
//! time range (inclusive start and end timestamps).

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Represents query parameters for retrieving events.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct EventQuery {
    // ---
    /// Optional event type to filter by.
    pub event_type: Option<String>,

    /// Optional start of time range (inclusive).
    pub start: Option<DateTime<Utc>>,

    /// Optional end of time range (inclusive).
    pub end: Option<DateTime<Utc>>,
}
