//! In-memory implementation of the EventRepository trait.
//!
//! Uses DashMap for concurrent, type-indexed event storage. Events are grouped
//! by event_type for efficient query filtering. This backend is suitable for testing
//! and non-persistent deployments.

use crate::domain::EventRepositoryPtr;
use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;

use crate::domain::{Event, EventQuery, EventRepository};

/// Creates an Arc-wrapped in-memory repository.
pub fn create() -> Result<EventRepositoryPtr> {
    // ---
    Ok(Arc::new(InMemoryEventRepository::new()))
}

/// A thread-safe, in-memory event repository using DashMap.
#[derive(Debug, Default)]
pub struct InMemoryEventRepository {
    /// Maps event_type → list of events
    store: DashMap<String, Vec<Event>>,
}

impl InMemoryEventRepository {
    /// Creates a new, empty in-memory repository.
    pub fn new() -> Self {
        Self {
            store: DashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl EventRepository for InMemoryEventRepository {
    // ---

    async fn store_event(&self, event: Event) -> anyhow::Result<()> {
        // ---

        self.store
            .entry(event.event_type.clone())
            .or_default()
            .push(event);
        Ok(())
    }

    async fn find_events(&self, query: EventQuery) -> anyhow::Result<Vec<Event>> {
        // ---

        let iter: Vec<Event> = match &query.event_type {
            Some(t) => self
                .store
                .get(t)
                .into_iter()
                .flat_map(|entry| entry.value().clone())
                .collect(),
            None => self
                .store
                .iter()
                .flat_map(|entry| entry.value().clone())
                .collect(),
        };

        let filtered = iter
            .into_iter()
            .filter(|event| {
                let ts = event.timestamp;
                match (query.start, query.end) {
                    (Some(start), Some(end)) => ts >= start && ts <= end,
                    (Some(start), None) => ts >= start,
                    (None, Some(end)) => ts <= end,
                    (None, None) => true,
                }
            })
            .collect();

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {

    // ---

    use super::*;
    use anyhow::Result;
    use chrono::DateTime;
    use chrono::Utc;
    use uuid::Uuid;

    // ---

    fn make_event(event_type: &str, timestamp: &str) -> Result<Event> {
        // ---

        let timestamp = DateTime::parse_from_rfc3339(timestamp)?.with_timezone(&Utc);

        Ok(Event {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            timestamp,
            payload: serde_json::json!({ "key": "value" }),
        })
    }

    // ---

    #[tokio::test]
    async fn store_and_fetch_event() -> Result<()> {
        // ---

        let repo = InMemoryEventRepository::new();
        let event = make_event("signup", "2025-06-16T12:00:00Z")?;
        repo.store_event(event.clone()).await?;

        let all = repo.find_events(EventQuery::default()).await?;
        anyhow::ensure!(all.len() == 1, "Expected 1 event, got {}", all.len());
        anyhow::ensure!(all[0].event_type == "signup");

        Ok(())
    }

    #[tokio::test]
    async fn filter_by_event_type() -> Result<()> {
        // ---

        let repo = InMemoryEventRepository::new();
        repo.store_event(make_event("signup", "2025-06-16T12:00:00Z")?)
            .await?;
        repo.store_event(make_event("login", "2025-06-16T12:05:00Z")?)
            .await?;

        let results = repo
            .find_events(EventQuery {
                event_type: Some("login".into()),
                start: None,
                end: None,
            })
            .await?;

        anyhow::ensure!(results.len() == 1);
        anyhow::ensure!(results[0].event_type == "login");

        Ok(())
    }

    #[tokio::test]
    async fn filter_by_time_range() -> Result<()> {
        // ---

        let repo = InMemoryEventRepository::new();
        repo.store_event(make_event("test", "2025-06-16T10:00:00Z")?)
            .await?;
        repo.store_event(make_event("test", "2025-06-16T11:00:00Z")?)
            .await?;
        repo.store_event(make_event("test", "2025-06-16T12:00:00Z")?)
            .await?;

        let start = DateTime::parse_from_rfc3339("2025-06-16T10:30:00Z")?.with_timezone(&Utc);
        let end = DateTime::parse_from_rfc3339("2025-06-16T11:30:00Z")?.with_timezone(&Utc);

        let results = repo
            .find_events(EventQuery {
                event_type: Some("test".into()),
                start: Some(start),
                end: Some(end),
            })
            .await?;

        anyhow::ensure!(results.len() == 1);
        anyhow::ensure!(
            results[0].timestamp
                == DateTime::parse_from_rfc3339("2025-06-16T11:00:00Z")?.with_timezone(&Utc)
        );

        Ok(())
    }

    #[tokio::test]
    async fn returns_empty_if_no_matches() -> Result<()> {
        // ---

        let repo = InMemoryEventRepository::new();
        let results = repo
            .find_events(EventQuery {
                event_type: Some("nonexistent".into()),
                start: None,
                end: None,
            })
            .await?;

        anyhow::ensure!(results.is_empty(), "Expected empty result set");

        Ok(())
    }
}
