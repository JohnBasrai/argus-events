//! Comprehensive integration tests for the Argus Events API.

use argus_events::{create_app, create_repository, Event};
use axum::Router;
use reqwest::Client;
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Helper to start a test server and return the base URL
async fn start_test_server() -> String {
    // ---
    
    let repo = create_repository("memory").expect("Failed to create memory repository");
    let app: Router = create_app(repo);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    format!("http://{}", addr)
}

#[tokio::test]
async fn get_events_returns_empty_list() -> anyhow::Result<()> {
    // ---

    let base_url = start_test_server().await;
    let client = Client::new();

    let response = client.get(&format!("{}/events", base_url)).send().await?;

    assert_eq!(response.status(), 200);
    let body = response.text().await?;
    assert_eq!(body, "[]");

    Ok(())
}

#[tokio::test]
async fn post_and_get_single_event() -> anyhow::Result<()> {
    // ---

    let base_url = start_test_server().await;
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
        .post(&format!("{}/events", base_url))
        .json(&event_payload)
        .send()
        .await?;

    assert_eq!(post_response.status(), 201);

    // Retrieve events
    let get_response = client.get(&format!("{}/events", base_url)).send().await?;
    assert_eq!(get_response.status(), 200);

    let events: Vec<Event> = get_response.json().await?;
    assert_eq!(events.len(), 1);
    
    let event = &events[0];
    assert_eq!(event.event_type, "user_signup");
    assert_eq!(event.payload["user_id"], "12345");
    assert_eq!(event.payload["email"], "test@example.com");

    Ok(())
}

#[tokio::test]
async fn post_multiple_events_and_filter_by_type() -> anyhow::Result<()> {
    // ---

    let base_url = start_test_server().await;
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
            .post(&format!("{}/events", base_url))
            .json(event)
            .send()
            .await?;
        assert_eq!(response.status(), 201);
    }

    // Get all events
    let all_response = client.get(&format!("{}/events", base_url)).send().await?;
    let all_events: Vec<Event> = all_response.json().await?;
    assert_eq!(all_events.len(), 4);

    // Filter by event type: user_signup
    let signup_response = client
        .get(&format!("{}/events?type=user_signup", base_url))
        .send()
        .await?;
    let signup_events: Vec<Event> = signup_response.json().await?;
    assert_eq!(signup_events.len(), 2);
    assert!(signup_events.iter().all(|e| e.event_type == "user_signup"));

    // Filter by event type: user_login
    let login_response = client
        .get(&format!("{}/events?type=user_login", base_url))
        .send()
        .await?;
    let login_events: Vec<Event> = login_response.json().await?;
    assert_eq!(login_events.len(), 1);
    assert_eq!(login_events[0].event_type, "user_login");

    Ok(())
}

#[tokio::test]
async fn filter_events_by_time_range() -> anyhow::Result<()> {
    // ---

    let base_url = start_test_server().await;
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
            .post(&format!("{}/events", base_url))
            .json(event)
            .send()
            .await?;
    }

    // Filter by time range: 10:30 to 12:30
    let filtered_response = client
        .get(&format!(
            "{}/events?type=test_event&start=2025-06-16T10:30:00Z&end=2025-06-16T12:30:00Z",
            base_url
        ))
        .send()
        .await?;

    let filtered_events: Vec<Event> = filtered_response.json().await?;
    assert_eq!(filtered_events.len(), 2);
    
    // Should include events at 11:00 and 12:00
    let sequences: Vec<i64> = filtered_events
        .iter()
        .map(|e| e.payload["sequence"].as_i64().unwrap())
        .collect();
    assert!(sequences.contains(&2));
    assert!(sequences.contains(&3));

    Ok(())
}

#[tokio::test]
async fn invalid_event_returns_400() -> anyhow::Result<()> {
    // ---

    let base_url = start_test_server().await;
    let client = Client::new();

    // Try to post invalid JSON
    let invalid_payload = json!({
        "event_type": "test",
        // Missing required timestamp field
        "payload": {"test": "data"}
    });

    let response = client
        .post(&format!("{}/events", base_url))
        .json(&invalid_payload)
        .send()
        .await?;

    // Should return 400 Bad Request or 422 Unprocessable Entity
    assert!(response.status() == 400 || response.status() == 422);

    Ok(())
}

#[tokio::test]
async fn events_have_unique_ids() -> anyhow::Result<()> {
    // ---

    let base_url = start_test_server().await;
    let client = Client::new();

    // Post identical events
    let event_payload = json!({
        "event_type": "duplicate_test",
        "timestamp": "2025-06-16T12:00:00Z",
        "payload": {"data": "same"}
    });

    for _ in 0..3 {
        let response = client
            .post(&format!("{}/events", base_url))
            .json(&event_payload)
            .send()
            .await?;
        assert_eq!(response.status(), 201);
    }

    // Retrieve all events
    let response = client.get(&format!("{}/events", base_url)).send().await?;
    let events: Vec<Event> = response.json().await?;

    assert_eq!(events.len(), 3);

    // All events should have unique IDs
    let mut ids = std::collections::HashSet::new();
    for event in &events {
        assert!(ids.insert(event.id), "Duplicate ID found: {}", event.id);
    }

    Ok(())
}