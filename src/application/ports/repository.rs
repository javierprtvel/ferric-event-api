use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::event::Event;

#[allow(dead_code)]
pub trait EventRepository {
    async fn find_all(&self) -> Vec<Event>;
    async fn find_by_id(&self, id: &Uuid) -> Option<Event>;
    fn find_by_title(&self, title: &str)
    -> impl std::future::Future<Output = Option<Event>> + Send;
    fn find_between(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: u64,
        offset: u64,
    ) -> impl std::future::Future<Output = Vec<Event>> + Send;
    fn save(&self, e: SaveEventRequest) -> impl std::future::Future<Output = Event> + Send;
    fn upsert(&self, entity: Event) -> impl std::future::Future<Output = Event> + Send;
}

#[derive(Clone)]
pub struct SaveEventRequest {
    pub title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub min_price: f64,
    pub max_price: f64,
}
