use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::application::ports::repository::{EventRepository, SaveEventRequest};
use crate::domain::event::Event;

pub struct InMemoryEventRepository(Mutex<HashMap<Uuid, Event>>);

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

    async fn find_between(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Vec<Event> {
        let event_store = self.0.lock().await;
        event_store
            .values()
            .filter(|e| start_time <= e.start_time && e.end_time <= end_time)
            .cloned()
            .collect()
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

pub struct PostgresEventRepository(PgPool);

impl PostgresEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
}

impl EventRepository for PostgresEventRepository {
    async fn find_all(&self) -> Vec<Event> {
        let query = sqlx::query_as!(
            PostgresEvent,
            r#"
            SELECT id, title, start_time, end_time, min_price as min_price_in_lowest_denomination, max_price as max_price_in_lowest_denomination
            FROM events
        "#
        );

        query
            .fetch_all(&self.0)
            .await
            .expect("Failed to find all events in event database")
            .iter()
            .map(|pe| pe.clone().into())
            .collect()
    }
    async fn find_between(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Vec<Event> {
        let query = sqlx::query_as!(
            PostgresEvent,
            r#"
            SELECT id, title, start_time, end_time, min_price as min_price_in_lowest_denomination, max_price as max_price_in_lowest_denomination
            FROM events
            WHERE start_time >= $1 AND end_time <= $2
        "#,
            start_time,
            end_time
        );

        query
            .fetch_all(&self.0)
            .await
            .expect("Failed to find events between datetimes in event database")
            .iter()
            .map(|pe| pe.clone().into())
            .collect()
    }
    async fn find_by_id(&self, id: &Uuid) -> Option<Event> {
        let query = sqlx::query_as!(
            PostgresEvent,
            r#"
                SELECT id, title, start_time, end_time, min_price as min_price_in_lowest_denomination, max_price as max_price_in_lowest_denomination
                FROM events
                WHERE id = $1
            "#,
            id,
        );

        query
            .fetch_optional(&self.0)
            .await
            .expect("Failed to find event by id in event database")
            .map(|pe| pe.into())
    }
    async fn find_by_title(&self, title: &str) -> Option<Event> {
        let query = sqlx::query_as!(
            PostgresEvent,
            r#"
            SELECT id, title, start_time, end_time, min_price as min_price_in_lowest_denomination, max_price as max_price_in_lowest_denomination
            FROM events
            WHERE title = $1 
        "#,
            title
        );

        query
            .fetch_optional(&self.0)
            .await
            .expect("Failed to find event by title in event database")
            .map(|pe| pe.into())
    }
    async fn save(&self, e: SaveEventRequest) -> Event {
        let event = PostgresEvent::from(e);
        let query = sqlx::query!(
            r#"
                INSERT INTO events (id, title, start_time, end_time, min_price, max_price)
                VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            event.id,
            event.title,
            event.start_time,
            event.end_time,
            event.min_price_in_lowest_denomination,
            event.max_price_in_lowest_denomination,
        );

        query
            .execute(&self.0)
            .await
            .expect("Failed to insert event record in event database");

        self.find_by_id(&event.id).await.unwrap()
    }
    async fn upsert(&self, entity: Event) -> Event {
        let event = PostgresEvent::from(entity);
        let query = sqlx::query!(
            r#"
                INSERT INTO events (id, title, start_time, end_time, min_price, max_price)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (id) DO UPDATE
                SET title = $2, start_time = $3, end_time = $4, min_price = $5, max_price = $6
            "#,
            event.id,
            event.title,
            event.start_time,
            event.end_time,
            event.min_price_in_lowest_denomination,
            event.max_price_in_lowest_denomination,
        );

        query
            .execute(&self.0)
            .await
            .expect("Failed to update event record in event database");

        self.find_by_id(&event.id).await.unwrap()
    }
}

#[derive(sqlx::FromRow, Clone)]
struct PostgresEvent {
    id: Uuid,
    title: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    min_price_in_lowest_denomination: i32,
    max_price_in_lowest_denomination: i32,
}

impl Into<Event> for PostgresEvent {
    fn into(self) -> Event {
        Event {
            id: self.id,
            title: self.title,
            start_time: self.start_time,
            end_time: self.end_time,
            min_price: self.min_price_in_lowest_denomination as f64 / 100.0,
            max_price: self.max_price_in_lowest_denomination as f64 / 100.0,
        }
    }
}

impl From<Event> for PostgresEvent {
    fn from(value: Event) -> Self {
        Self {
            id: value.id,
            title: value.title,
            start_time: value.start_time,
            end_time: value.end_time,
            min_price_in_lowest_denomination: (value.min_price * 100.0) as i32,
            max_price_in_lowest_denomination: (value.max_price * 100.0) as i32,
        }
    }
}

impl From<SaveEventRequest> for PostgresEvent {
    fn from(value: SaveEventRequest) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: value.title,
            start_time: value.start_time,
            end_time: value.end_time,
            min_price_in_lowest_denomination: (value.min_price * 100.0) as i32,
            max_price_in_lowest_denomination: (value.max_price * 100.0) as i32,
        }
    }
}
