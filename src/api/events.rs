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
use std::time::Instant;
use tracing::info;
use uuid::Uuid;

use crate::domain::{Event, EventQuery, EventRepositoryPtr};
use crate::MetricsPtr;

/// Request body for `POST /events`
#[derive(Debug, Deserialize)]
pub struct EventInput {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
}

/// Application state containing shared resources
#[derive(Clone)]
pub struct AppState {
    pub repo: EventRepositoryPtr,
    pub metrics: MetricsPtr,
}

/// POST /events handler
pub async fn submit_event(
    State(state): State<AppState>,
    Json(input): Json<EventInput>,
) -> impl IntoResponse {
    // ---

    let start = Instant::now();

    let event = Event {
        id: Uuid::new_v4(),
        event_type: input.event_type.clone(),
        timestamp: input.timestamp,
        payload: input.payload,
    };

    tracing::info!(
        event_type = %input.event_type,
        event_id = %event.id,
        "Processing event submission"
    );

    match state.repo.store_event(event).await {
        Ok(_) => {
            info!(
                event_type = %input.event_type,
                "Event stored successfully"
            );
            state.metrics.record_event_created();
            state
                .metrics
                .record_http_request(start, "/events", "POST", 201);
            StatusCode::CREATED
        }
        Err(err) => {
            tracing::error!(
                ?err,
                event_type = %input.event_type,
                "Failed to store event"
            );
            state
                .metrics
                .record_http_request(start, "/events", "POST", 500);
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
    State(state): State<AppState>,
    Query(params): Query<GetEventsQuery>,
) -> impl IntoResponse {
    // ---

    let start = Instant::now();

    tracing::debug!(
        event_type = ?params.event_type,
        start_time = ?params.start,
        end_time = ?params.end,
        "Processing events query"
    );

    let query = match parse_query(params) {
        Ok(q) => q,
        Err(e) => {
            tracing::warn!(?e, "Invalid query parameters");
            state
                .metrics
                .record_http_request(start, "/events", "GET", 400);
            return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
        }
    };

    match state.repo.find_events(query).await {
        Ok(events) => {
            tracing::info!(event_count = events.len(), "Successfully retrieved events");
            state
                .metrics
                .record_http_request(start, "/events", "GET", 200);
            Json(events).into_response()
        }
        Err(e) => {
            tracing::error!(?e, "Failed to retrieve events");
            state
                .metrics
                .record_http_request(start, "/events", "GET", 500);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// GET /metrics handler - Prometheus metrics endpoint
async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    // ---

    tracing::debug!("Serving metrics endpoint");

    let metrics_content = state.metrics.render();

    (
        StatusCode::OK,
        [("content-type", "text/plain; charset=utf-8")],
        metrics_content,
    )
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

/// Creates the router with event-related routes and metrics endpoint.
pub fn event_routes(repo: EventRepositoryPtr, metrics: MetricsPtr) -> Router {
    // ---

    let state = AppState { repo, metrics };

    Router::new()
        .route("/events", post(submit_event))
        .route("/events", get(get_events))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}
