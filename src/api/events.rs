//! HTTP handlers for event-related endpoints.
//!
//! This file defines routes for submitting and querying events via Axum.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::domain::{Event, EventQuery, EventRepositoryPtr};

/// Request body for `POST /events`
#[derive(Debug, Deserialize)]
pub struct EventInput {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
}

/// POST /events handler
pub async fn submit_event(
    State(repo): State<EventRepositoryPtr>,
    Json(input): Json<EventInput>,
) -> impl IntoResponse {
    // ---

    let event = Event {
        id: Uuid::new_v4(),
        event_type: input.event_type,
        timestamp: input.timestamp,
        payload: input.payload,
    };

    match repo.store_event(event).await {
        Ok(_) => {
            info!("Event stored successfully");
            StatusCode::CREATED
        }
        Err(err) => {
            tracing::error!(?err, "Failed to store event");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Query parameters for GET /events
#[derive(Debug, Deserialize)]
pub struct GetEventsQuery {
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
}

/// GET /events handler
async fn get_events(
    State(repo): State<EventRepositoryPtr>,
    Query(params): Query<GetEventsQuery>,
) -> impl IntoResponse {
    // ---

    let query = match parse_query(params) {
        Ok(q) => q,
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };

    match repo.find_events(query).await {
        Ok(events) => Json(events).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// Parse query parameters into EventQuery
fn parse_query(params: GetEventsQuery) -> anyhow::Result<EventQuery> {
    // ---

    let start = match params.start {
        Some(s) => Some(chrono::DateTime::parse_from_rfc3339(&s)?.with_timezone(&chrono::Utc)),
        None => None,
    };

    let end = match params.end {
        Some(e) => Some(chrono::DateTime::parse_from_rfc3339(&e)?.with_timezone(&chrono::Utc)),
        None => None,
    };

    Ok(EventQuery {
        event_type: params.event_type,
        start,
        end,
    })
}

/// Creates the router with event-related routes.
pub fn event_routes(repo: EventRepositoryPtr) -> Router {
    // ---

    Router::new()
        .route("/events", post(submit_event))
        .route("/events", get(get_events))
        .with_state(repo)
}
