use crate::domain::{Event, EventQuery, EventRepository, EventRepositoryPtr};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct NoopRepository;

#[async_trait]
impl EventRepository for NoopRepository {
    async fn store_event(&self, _event: Event) -> Result<()> {
        tracing::info!("NoopRepository: store_event called");
        Ok(())
    }

    async fn find_events(&self, _query: EventQuery) -> Result<Vec<Event>> {
        tracing::info!("NoopRepository: find_events called");
        Ok(vec![])
    }
}

pub fn create() -> Result<EventRepositoryPtr> {
    Ok(Arc::new(NoopRepository))
}
