use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::ports::repository::{EventRepository, SaveEventRequest};
use crate::domain::event::Event;

#[allow(dead_code)]
pub struct FailingEventRepository;

#[allow(unused_variables)]
impl EventRepository for FailingEventRepository {
    async fn find_all(&self) -> Result<Vec<Event>> {
        todo!("Not yet implemented")
    }
    async fn find_between(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Event>> {
        anyhow::bail!("Failed to find events between datetimes in event database")
    }
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Event>> {
        todo!("Not yet implemented")
    }
    async fn find_by_title(&self, title: &str) -> Result<Option<Event>> {
        todo!("Not yet implemented")
    }
    async fn save(&self, e: SaveEventRequest) -> Result<Event> {
        todo!("Not yet implemented")
    }
    async fn upsert(&self, entity: Event) -> Result<Event> {
        todo!("Not yet implemented")
    }
}
