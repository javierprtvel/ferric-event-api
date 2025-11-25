use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::ports::repository::{EventRepository, SaveEventRequest};
use crate::domain::event::Event;

#[allow(dead_code)]
pub struct DummyEventRepository;

#[allow(unused_variables)]
impl EventRepository for DummyEventRepository {
    async fn find_all(&self) -> Vec<Event> {
        todo!("Not yet implemented")
    }
    async fn find_between(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Vec<Event> {
        todo!("Not yet implemented")
    }
    async fn find_by_id(&self, id: &Uuid) -> Option<Event> {
        todo!("Not yet implemented")
    }
    async fn find_by_title(&self, title: &str) -> Option<Event> {
        todo!("Not yet implemented")
    }
    async fn save(&self, e: SaveEventRequest) -> Event {
        todo!("Not yet implemented")
    }
    async fn upsert(&self, entity: Event) -> Event {
        todo!("Not yet implemented")
    }
}
