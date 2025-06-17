//! Comprehensive integration tests for the Argus Events API.

use anyhow::{anyhow, ensure, Context, Result};
use argus_events::{create_app, create_metrics, create_repository, Event};
use axum::Router;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::json;
use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Helper to start a test server and return the base URL
///
/// This function provides environment-aware test server selection:
/// - **Container mode**: When ARGUS_TEST_USE_CONTAINERS is set, delegates to
///   `start_container_server()` for isolated container-per-test execution
/// - **Local mode**: Creates an embedded test server using ephemeral ports
///   (127.0.0.1:0) for fast local development and debugging
///
/// The ephemeral port binding (port 0) lets the OS assign an available port,
/// preventing conflicts when multiple test processes run simultaneously.
async fn start_test_server() -> Result<String> {
    // ---

    // Check if we should use external containers (set by build.sh)
    if std::env::var("ARGUS_TEST_USE_CONTAINERS").is_ok() {
        return start_container_server().await;
    }

    // Otherwise, start our own test server (for local development)
    let repo = create_repository("memory").expect("Failed to create memory repository");
    let metrics = create_metrics()?;
    let app: Router = create_app(repo, metrics)?;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    Ok(format!("http://{}", addr))
}

/// Start a fresh container for this test and return the base URL
///
/// This function demonstrates production-quality container testing by:
/// - Creating a unique container name using process ID + random number
/// - Using Docker's ephemeral port mapping (0:3000) to avoid port conflicts
/// - Waiting for the container to be ready before returning the URL
/// - Scheduling automatic cleanup when the test completes
///
/// The random port allocation ensures multiple tests can run concurrently
/// without interfering with each other, even in CI environments.
pub async fn start_container_server() -> Result<String> {
    // ---
    let image_name =
        std::env::var("ARGUS_TEST_IMAGE").unwrap_or_else(|_| "argus-events:latest".to_string());
    let container_name = format!(
        "argus-test-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    );

    // Start container with random port
    let output = Command::new("docker")
        .args([
            "run",
            "-d",
            "--name",
            &container_name,
            "-p",
            "0:3000",
            &image_name,
        ])
        .output()
        .with_context(|| "Failed to start Docker container")?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failed to start container: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Get the mapped port
    let port_output = Command::new("docker")
        .args(["port", &container_name, "3000/tcp"])
        .output()
        .with_context(|| "Failed to get container port")?;

    let port_str = String::from_utf8(port_output.stdout).context("Port output not valid UTF-8")?;
    let mapped_port = port_str
        .trim()
        .split(':')
        .next_back()
        .ok_or_else(|| anyhow!("Unexpected port mapping format: {}", port_str))?;

    let base_url = format!("http://127.0.0.1:{}", mapped_port);

    // Wait up to 10 seconds for the server to be ready
    for _ in 0..20 {
        if let Ok(response) = reqwest::get(&format!("{}/events", base_url)).await {
            if response.status().is_success() {
                break;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Cleanup container when done
    let cleanup_container = container_name.clone();
    tokio::spawn(async move {
        let _ = Command::new("docker")
            .args(["stop", &cleanup_container])
            .output();
        let _ = Command::new("docker")
            .args(["rm", &cleanup_container])
            .output();
    });

    Ok(base_url)
}

// Helper function to generate random numbers (simple implementation)
mod rand {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn random<T>() -> T
    where
        T: From<u64>,
    {
        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        T::from(hasher.finish())
    }
}

#[tokio::test]
async fn get_events_returns_empty_list() -> anyhow::Result<()> {
    // ---

    let base_url = start_test_server().await?;
    let client = Client::new();

    let response = client.get(format!("{}/events", base_url)).send().await?;

    ensure!(response.status() == 200, "Expected status 200, got {}", response.status());
    let body = response.text().await?;
    ensure!(body == "[]", "Expected empty array '[]', got '{}'", body);

    Ok(())
}

#[tokio::test]
async fn post_and_get_single_event() -> anyhow::Result<()> {
    // ---

    let base_url = start_test_server().await?;
    let client = Client::new();

    // Post an event
    let event_payload = json!({
        "event_type": "user_signup",
        "timestamp": "2025-06-16T12:00:00Z",
        "payload": {
            "user_id": "12345",
            "email": "test@example.com",
            "source": "web"
        }
    });

    let post_response = client
        .post(format!("{}/events", base_url))
        .json(&event_payload)
        .send()
        .await?;

    ensure!(post_response.status() == 201, "Expected status 201, got {}", post_response.status());

    // Retrieve events
    let get_response = client.get(format!("{}/events", base_url)).send().await?;
    ensure!(get_response.status() == 200, "Expected status 200, got {}", get_response.status());

    let events: Vec<Event> = get_response.json().await?;
    ensure!(events.len() == 1, "Expected 1 event, got {}", events.len());

    let event = &events[0];
    ensure!(event.event_type == "user_signup", "Expected event_type 'user_signup', got '{}'", event.event_type);
    ensure!(event.payload["user_id"] == "12345", "Expected user_id '12345', got '{}'", event.payload["user_id"]);
    ensure!(event.payload["email"] == "test@example.com", "Expected email 'test@example.com', got '{}'", event.payload["email"]);

    Ok(())
}

/// Assignment Requirement: GET /events?type=xyz - Filter by event type
/// This test validates multiple event types with filtering validation
#[tokio::test]
async fn post_multiple_events_and_filter_by_type() -> anyhow::Result<()> {
    // ---

    let base_url = start_test_server().await?;
    let client = Client::new();

    // Post multiple events of different types
    let events = vec![
        json!({
            "event_type": "user_signup",
            "timestamp": "2025-06-16T12:00:00Z",
            "payload": {"user_id": "1"}
        }),
        json!({
            "event_type": "user_login",
            "timestamp": "2025-06-16T12:05:00Z",
            "payload": {"user_id": "1"}
        }),
        json!({
            "event_type": "user_signup",
            "timestamp": "2025-06-16T12:10:00Z",
            "payload": {"user_id": "2"}
        }),
        json!({
            "event_type": "page_view",
            "timestamp": "2025-06-16T12:15:00Z",
            "payload": {"page": "/dashboard"}
        }),
    ];

    // Post all events
    for event in &events {
        let response = client
            .post(format!("{}/events", base_url))
            .json(event)
            .send()
            .await?;
        ensure!(response.status() == 201, "Failed to post event, got status {}", response.status());
    }

    // Get all events
    let all_response = client.get(format!("{}/events", base_url)).send().await?;
    let all_events: Vec<Event> = all_response.json().await?;
    ensure!(all_events.len() == 4, "Expected 4 total events, got {}", all_events.len());

    // Filter by event type: user_signup
    let signup_response = client
        .get(format!("{}/events?type=user_signup", base_url))
        .send()
        .await?;
    let signup_events: Vec<Event> = signup_response.json().await?;
    ensure!(signup_events.len() == 2, "Expected 2 user_signup events, got {}", signup_events.len());
    ensure!(signup_events.iter().all(|e| e.event_type == "user_signup"), "Not all events are user_signup type");

    // Filter by event type: user_login
    let login_response = client
        .get(format!("{}/events?type=user_login", base_url))
        .send()
        .await?;
    let login_events: Vec<Event> = login_response.json().await?;
    ensure!(login_events.len() == 1, "Expected 1 user_login event, got {}", login_events.len());
    ensure!(login_events[0].event_type == "user_login", "Expected user_login, got {}", login_events[0].event_type);

    Ok(())
}

/// Assignment Requirement: GET /events?start=...&end=... - Filter by time range  
/// This test provides additional time range scenarios and edge cases
#[tokio::test]
async fn filter_events_by_time_range() -> anyhow::Result<()> {
    // ---

    let base_url = start_test_server().await?;
    let client = Client::new();

    // Post events at different times
    let events = vec![
        json!({
            "event_type": "test_event",
            "timestamp": "2025-06-16T10:00:00Z",
            "payload": {"sequence": 1}
        }),
        json!({
            "event_type": "test_event",
            "timestamp": "2025-06-16T11:00:00Z",
            "payload": {"sequence": 2}
        }),
        json!({
            "event_type": "test_event",
            "timestamp": "2025-06-16T12:00:00Z",
            "payload": {"sequence": 3}
        }),
        json!({
            "event_type": "test_event",
            "timestamp": "2025-06-16T13:00:00Z",
            "payload": {"sequence": 4}
        }),
    ];

    for event in &events {
        client
            .post(format!("{}/events", base_url))
            .json(event)
            .send()
            .await?;
    }

    // Filter by time range: 10:30 to 12:30
    let filtered_response = client
        .get(format!(
            "{}/events?type=test_event&start=2025-06-16T10:30:00Z&end=2025-06-16T12:30:00Z",
            base_url
        ))
        .send()
        .await?;

    let filtered_events: Vec<Event> = filtered_response.json().await?;
    ensure!(filtered_events.len() == 2, "Expected 2 filtered events, got {}", filtered_events.len());

    // Should include events at 11:00 and 12:00
    let sequences: Vec<i64> = filtered_events
        .iter()
        .map(|e| e.payload["sequence"].as_i64().unwrap())
        .collect();
    ensure!(sequences.contains(&2), "Missing sequence 2 in results: {:?}", sequences);
    ensure!(sequences.contains(&3), "Missing sequence 3 in results: {:?}", sequences);

    Ok(())
}

/// Assignment Requirement: Error handling - Malformed request validation
/// Validates proper HTTP status codes for all scenarios
#[tokio::test]
async fn invalid_event_returns_400() -> anyhow::Result<()> {
    // ---

    let base_url = start_test_server().await?;
    let client = Client::new();

    // Try to post invalid JSON
    let invalid_payload = json!({
        "event_type": "test",
        // Missing required timestamp field
        "payload": {"test": "data"}
    });

    let response = client
        .post(format!("{}/events", base_url))
        .json(&invalid_payload)
        .send()
        .await?;

    // Should return 400 Bad Request or 422 Unprocessable Entity
    ensure!(response.status() == 400 || response.status() == 422, 
           "Expected 400 or 422 for invalid payload, got {}", response.status());

    Ok(())
}

#[tokio::test]
async fn events_have_unique_ids() -> anyhow::Result<()> {
    // ---

    let base_url = start_test_server().await?;
    let client = Client::new();

    // Post identical events
    let event_payload = json!({
        "event_type": "duplicate_test",
        "timestamp": "2025-06-16T12:00:00Z",
        "payload": {"data": "same"}
    });

    for _ in 0..3 {
        let response = client
            .post(format!("{}/events", base_url))
            .json(&event_payload)
            .send()
            .await?;
        ensure!(response.status() == 201, "Failed to post duplicate event, got status {}", response.status());
    }

    // Retrieve all events
    let response = client.get(format!("{}/events", base_url)).send().await?;
    let events: Vec<Event> = response.json().await?;

    ensure!(events.len() == 3, "Expected 3 duplicate events, got {}", events.len());

    // All events should have unique IDs
    let mut ids = std::collections::HashSet::new();
    for event in &events {
        ensure!(ids.insert(event.id), "Duplicate ID found: {}", event.id);
    }

    Ok(())
}

// Begin query_param tests

// Helper macro for posting events and verifying success
macro_rules! post_events {
    // ----
    ($app:expr, $($event:expr),+) => {
        {
            $(
                let response = $app.post_event($event.clone()).await;
                ensure!(response.status() == 201, "Failed to post event: status {}", response.status());
            )+
        }
    };
}

// Helper macro for asserting response status and extracting JSON array
macro_rules! get_events_array {
    ($response:expr) => {{
        ensure!(
            $response.status() == 200,
            "Request failed with status {}",
            $response.status()
        );
        let events: serde_json::Value = $response.json().await?;
        events.as_array().unwrap().clone() // Clone to avoid lifetime issues
    }};
}

// Helper macro for extracting field values from event arrays
macro_rules! extract_field {
    // ----
    ($events:expr, $field_path:expr, $field_type:ty) => {{
        let result: Result<Vec<$field_type>> = $events
            .iter()
            .map(|e| {
                let value = $field_path(e);
                value.ok_or_else(|| anyhow::anyhow!("Event missing expected field"))
            })
            .collect();
        result?
    }};
}

// Helper functions for creating test events
fn create_signup_event(timestamp: &str, user_id: &str, email: &str) -> serde_json::Value {
    // ----
    json!({
        "event_type": "user_signup",
        "timestamp": timestamp,
        "payload": {
            "user_id": user_id,
            "email": email
        }
    })
}

fn create_purchase_event(timestamp: &str, user_id: &str, amount: f64) -> serde_json::Value {
    // ----
    json!({
        "event_type": "purchase",
        "timestamp": timestamp,
        "payload": {
            "user_id": user_id,
            "amount": amount
        }
    })
}

fn create_test_event_with_sequence(
    timestamp: &str,
    user_id: &str,
    sequence: i64,
) -> serde_json::Value {
    // ----
    json!({
        "event_type": "test_event",
        "timestamp": timestamp,
        "payload": {
            "user_id": user_id,
            "sequence": sequence
        }
    })
}

// Helper function for validating time range
fn validate_time_range(events: &[serde_json::Value], start: &str, end: &str) -> Result<()> {
    // ----
    let start_time: DateTime<Utc> = start.parse()?;
    let end_time: DateTime<Utc> = end.parse()?;

    for event in events {
        let timestamp_str = event["timestamp"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Event missing timestamp field"))?;
        let timestamp: DateTime<Utc> = timestamp_str.parse()?;

        ensure!(
            timestamp >= start_time,
            "Event timestamp {} is before start time {}",
            timestamp,
            start_time
        );
        ensure!(
            timestamp <= end_time,
            "Event timestamp {} is after end time {}",
            timestamp,
            end_time
        );
    }
    Ok(())
}

// Helper function for validating event types
fn validate_event_types(events: &[serde_json::Value], expected_type: &str) -> Result<()> {
    // ----

    for event in events {
        let event_type = event["event_type"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Event missing event_type field"))?;
        ensure!(
            event_type == expected_type,
            "Expected event_type '{}', got '{}'",
            expected_type,
            event_type
        );
    }
    Ok(())
}

/// Assignment Requirement: GET /events?type=xyz - Filter by event type
/// Core type filtering functionality with comprehensive test coverage
#[tokio::test]
async fn test_get_events_filter_by_type() -> Result<()> {
    // ---

    let app = spawn_app().await;

    // Create test events with different types
    let signup1 = create_signup_event("2024-01-15T10:30:00Z", "user123", "test1@example.com");
    let purchase = create_purchase_event("2024-01-15T11:00:00Z", "user456", 29.99);
    let signup2 = create_signup_event("2024-01-15T12:00:00Z", "user789", "test2@example.com");

    post_events!(app, signup1, purchase, signup2);

    // Test filtering by event type
    let response = app.get_events_with_query("type=user_signup").await;
    let events_array = get_events_array!(response);

    // Should return only user_signup events
    ensure!(
        events_array.len() == 2,
        "Expected 2 user_signup events, got {}",
        events_array.len()
    );

    validate_event_types(&events_array, "user_signup")?;

    // Verify the correct events were returned
    let user_ids = extract_field!(
        events_array,
        |err: &serde_json::Value| err["payload"]["user_id"].as_str().map(|s| s.to_string()),
        String
    );
    ensure!(
        user_ids.contains(&"user123".to_string()),
        "Missing expected user_id 'user123'"
    );
    ensure!(
        user_ids.contains(&"user789".to_string()),
        "Missing expected user_id 'user789'"
    );

    Ok(())
}

/// Assignment Requirement: GET /events?start=...&end=... - Filter by time range
/// Time range filtering with boundary validation and inclusive testing
#[tokio::test]
async fn test_get_events_filter_by_time_range() -> Result<()> {
    // ---

    let app = spawn_app().await;

    // Create test events with different timestamps
    let events = [
        create_test_event_with_sequence("2024-01-10T10:00:00Z", "user1", 1),
        create_test_event_with_sequence("2024-01-15T10:00:00Z", "user2", 2),
        create_test_event_with_sequence("2024-01-20T10:00:00Z", "user3", 3),
        create_test_event_with_sequence("2024-01-25T10:00:00Z", "user4", 4),
    ];

    post_events!(app, events[0], events[1], events[2], events[3]);

    // Test filtering by time range (inclusive)
    let start = "2024-01-12T00:00:00Z";
    let end = "2024-01-22T00:00:00Z";
    let query = format!("start={}&end={}", start, end);

    let response = app.get_events_with_query(&query).await;
    let events_array = get_events_array!(response);

    // Should return events 2 and 3 (within the time range)
    ensure!(
        events_array.len() == 2,
        "Expected 2 events in time range, got {}",
        events_array.len()
    );

    let sequences = extract_field!(
        events_array,
        |err: &serde_json::Value| err["payload"]["sequence"].as_i64(),
        i64
    );
    ensure!(sequences.contains(&2), "Missing expected sequence 2");
    ensure!(sequences.contains(&3), "Missing expected sequence 3");
    ensure!(
        !sequences.contains(&1),
        "Sequence 1 should be filtered out (before start)"
    );
    ensure!(
        !sequences.contains(&4),
        "Sequence 4 should be filtered out (after end)"
    );

    Ok(())
}

/// Assignment Requirement: GET /events?type=xyz&start=...&end=... - Combined filters
/// Type + time range filtering working correctly together
#[tokio::test]
async fn test_get_events_combined_filters() -> Result<()> {
    // ---

    let app = spawn_app().await;

    // Create test events with mixed types and timestamps
    let events = [
        create_signup_event("2024-01-10T10:00:00Z", "user1", "early@example.com"),
        create_purchase_event("2024-01-15T10:00:00Z", "user2", 19.99),
        create_signup_event("2024-01-15T11:00:00Z", "user3", "middle@example.com"),
        create_signup_event("2024-01-20T10:00:00Z", "user4", "late@example.com"),
        create_purchase_event("2024-01-20T11:00:00Z", "user5", 49.99),
    ];

    post_events!(app, events[0], events[1], events[2], events[3], events[4]);

    // Test combined filters: type=user_signup AND time range
    let start = "2024-01-12T00:00:00Z";
    let end = "2024-01-22T00:00:00Z";
    let query = format!("type=user_signup&start={}&end={}", start, end);

    let response = app.get_events_with_query(&query).await;
    let events_array = get_events_array!(response);

    // Should return only user_signup events within the time range
    ensure!(
        events_array.len() == 2,
        "Expected 2 events with combined filters, got {}",
        events_array.len()
    );

    validate_event_types(&events_array, "user_signup")?;
    validate_time_range(&events_array, start, end)?;

    // Verify we got the correct events
    let emails = extract_field!(
        events_array,
        |err: &serde_json::Value| err["payload"]["email"].as_str().map(|s| s.to_string()),
        String
    );
    ensure!(
        emails.contains(&"middle@example.com".to_string()),
        "Missing expected email 'middle@example.com'"
    );
    ensure!(
        emails.contains(&"late@example.com".to_string()),
        "Missing expected email 'late@example.com'"
    );
    ensure!(
        !emails.contains(&"early@example.com".to_string()),
        "Email 'early@example.com' should be filtered out (outside time range)"
    );

    Ok(())
}

/// Assignment Requirement: Edge cases - Empty results, URL encoding, case sensitivity
/// No matching results scenarios testing
#[tokio::test]
async fn test_get_events_empty_results_with_filters() -> Result<()> {
    // ---

    let app = spawn_app().await;

    let event = create_signup_event("2024-01-15T10:00:00Z", "user123", "test@example.com");
    post_events!(app, event);

    // Test filter that should return no results
    let response = app.get_events_with_query("type=nonexistent_type").await;
    let events_array = get_events_array!(response);
    ensure!(
        events_array.len() == 0,
        "Expected 0 events for nonexistent type, got {}",
        events_array.len()
    );

    // Test time range that should return no results
    let start = "2024-02-01T00:00:00Z";
    let end = "2024-02-28T23:59:59Z";
    let query = format!("start={}&end={}", start, end);

    let response = app.get_events_with_query(&query).await;
    let events_array = get_events_array!(response);
    ensure!(
        events_array.len() == 0,
        "Expected 0 events for future time range, got {}",
        events_array.len()
    );

    Ok(())
}

/// Assignment Requirement: Input validation - Invalid timestamps return HTTP 400
/// Time range validation: start >= end returns HTTP 400 (discovered via TDD)
/// Invalid date formats and time range validation
#[tokio::test]
async fn test_get_events_invalid_query_parameters() -> Result<()> {
    // ---

    let app = spawn_app().await;

    // Helper function for testing error cases with lifetime annotation
    let test_error_case = |app: Arc<TestApp>, query: &'static str, description: &'static str| async move {
        let response = app.get_events_with_query(query).await;
        ensure!(
            response.status() == 400,
            "Expected 400 for {}, got {}",
            description,
            response.status()
        );
        Ok::<(), anyhow::Error>(())
    };

    test_error_case(app.clone(), "start=invalid-date", "invalid start date").await?;
    test_error_case(app.clone(), "end=not-a-timestamp", "invalid end date").await?;

    // Note: This test validates start < end requirement (discovered via TDD)
    test_error_case(
        app.clone(),
        "start=2024-01-20T00:00:00Z&end=2024-01-10T00:00:00Z",
        "invalid time range (start > end)",
    )
    .await?;

    Ok(())
}

/// Assignment Requirement: Edge cases - Empty results, URL encoding, case sensitivity
/// URL encoding, empty params, case sensitivity testing
#[tokio::test]
async fn test_get_events_query_parameter_edge_cases() -> Result<()> {
    // ---

    let app = spawn_app().await;

    let event = json!({
        "event_type": "test_event",
        "timestamp": "2024-01-15T10:00:00Z",
        "payload": {
            "user_id": "user123",
            "test": true
        }
    });

    post_events!(app, event);

    // Helper function for testing edge cases with lifetime annotation
    let test_query_case = |app: Arc<TestApp>,
                           query: &'static str,
                           expected_count: usize,
                           description: &'static str| async move {
        let response = app.get_events_with_query(query).await;
        let events_array = get_events_array!(response);
        ensure!(
            events_array.len() == expected_count,
            "Expected {} events for {}, got {}",
            expected_count,
            description,
            events_array.len()
        );
        Ok::<(), anyhow::Error>(())
    };

    test_query_case(app.clone(), "type=", 0, "empty type").await?;
    test_query_case(app.clone(), "type=test%5Fevent", 1, "URL encoded type").await?; // "test_event" URL encoded
    test_query_case(
        app.clone(),
        "type=TEST_EVENT",
        0,
        "case-mismatched type (assuming case-sensitive)",
    )
    .await?;

    Ok(())
}

/// Test application wrapper for easier testing
pub struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
}

impl TestApp {
    async fn post_event(&self, event: serde_json::Value) -> reqwest::Response {
        self.client
            .post(&format!("{}/events", &self.address))
            .json(&event)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    async fn get_events_with_query(&self, query: &str) -> reqwest::Response {
        self.client
            .get(&format!("{}/events?{}", &self.address, query))
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

/// Spawn a test application instance and return TestApp wrapper
async fn spawn_app() -> Arc<TestApp> {
    let base_url = start_test_server()
        .await
        .expect("Failed to start test server");

    Arc::new(TestApp {
        address: base_url,
        client: reqwest::Client::new(),
    })
}
