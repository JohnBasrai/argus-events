//! Domain model representing an individual event in the system.
//!
//! This struct is used throughout the application to represent
//! user-submitted events, including their type, timestamp, and
//! arbitrary JSON payload data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a single event submitted to the tracking system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    // ---
    /// Unique identifier for the event.
    pub id: Uuid,

    /// The type/category of the event (e.g., "user_signup").
    pub event_type: String,

    /// Timestamp when the event occurred, in UTC.
    pub timestamp: DateTime<Utc>,

    /// Arbitrary structured payload data associated with the event.
    pub payload: serde_json::Value,
}
