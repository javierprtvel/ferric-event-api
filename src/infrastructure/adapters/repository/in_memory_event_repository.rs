use std::collections::HashMap;

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::application::ports::repository::{EventRepository, SaveEventRequest};
use crate::domain::event::Event;

pub struct InMemoryEventRepository(Mutex<HashMap<Uuid, Event>>);

#[allow(dead_code)]
impl InMemoryEventRepository {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

#[allow(dead_code)]
impl EventRepository for InMemoryEventRepository {
    async fn find_all(&self) -> Vec<Event> {
        let event_store = self.0.lock().await;
        event_store.values().cloned().collect()
    }

    async fn find_by_id(&self, id: &Uuid) -> Option<Event> {
        let event_store = self.0.lock().await;
        event_store.get(id).cloned()
    }

    async fn find_by_title(&self, title: &str) -> Option<Event> {
        let event_store = self.0.lock().await;
        event_store.values().find(|e| e.title == title).cloned()
    }

    async fn find_between(
        &self,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
        _limit: u64,
        _offset: u64,
    ) -> Vec<Event> {
        todo!("Implement pagination")
    }

    async fn save(&self, e: SaveEventRequest) -> Event {
        let event = Event {
            id: Uuid::new_v4(),
            title: e.title,
            start_time: e.start_time,
            end_time: e.end_time,
            min_price: e.min_price,
            max_price: e.max_price,
        };

        let mut event_store = self.0.lock().await;
        event_store.insert(event.id, event.clone());

        event
    }

    async fn upsert(&self, entity: Event) -> Event {
        let mut event_store = self.0.lock().await;
        event_store.insert(entity.id, entity.clone());
        entity
    }
}
