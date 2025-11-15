use std::collections::HashMap;

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub min_price: f64,
    pub max_price: f64,
}

pub struct EventRepository(Mutex<HashMap<Uuid, Event>>);

#[allow(dead_code)]
impl EventRepository {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }

    pub async fn find_all(&self) -> Vec<Event> {
        let event_store = self.0.lock().await;
        event_store.values().cloned().collect()
    }

    pub async fn find_by_id(&self, id: &Uuid) -> Option<Event> {
        let event_store = self.0.lock().await;
        event_store.get(id).cloned()
    }

    pub async fn find_by_title(&self, title: &str) -> Option<Event> {
        let event_store = self.0.lock().await;
        event_store.values().find(|e| e.title == title).cloned()
    }

    pub async fn find_between(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Vec<Event> {
        let event_store = self.0.lock().await;
        event_store
            .values()
            .filter(|e| start_time <= e.start_time && e.end_time <= end_time)
            .cloned()
            .collect()
    }

    pub async fn save(&self, e: SaveEventRequest) -> Event {
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

    pub async fn upsert(&self, entity: Event) -> Event {
        let mut event_store = self.0.lock().await;
        event_store.insert(entity.id, entity.clone());
        entity
    }
}

#[derive(Clone)]
pub struct SaveEventRequest {
    pub title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub min_price: f64,
    pub max_price: f64,
}
