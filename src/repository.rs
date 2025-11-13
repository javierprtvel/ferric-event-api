use std::collections::HashMap;

use chrono::{DateTime, Utc};
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

#[derive(Clone)]
pub struct EventRepository(HashMap<Uuid, Event>);

#[allow(dead_code)]
impl EventRepository {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn find_all(&self) -> Vec<Event> {
        let Self(event_store) = self;
        event_store.values().cloned().collect()
    }

    pub fn find_by_id(&self, id: Uuid) -> Option<&Event> {
        let Self(event_store) = self;
        event_store.get(&id)
    }

    pub fn find_between(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Vec<&Event> {
        let Self(event_store) = self;
        event_store
            .values()
            .filter(|e| start_time <= e.start_time && e.end_time <= end_time)
            .collect()
    }

    pub fn save(&mut self, e: SaveEventRequest) -> Event {
        let Self(event_store) = self;

        let event = Event {
            id: Uuid::new_v4(),
            title: e.title,
            start_time: e.start_time,
            end_time: e.end_time,
            min_price: e.min_price,
            max_price: e.max_price,
        };
        event_store.insert(event.id, event.clone());

        event
    }

    pub fn upsert(&mut self, entity: Event) -> Event {
        let Self(event_store) = self;
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
