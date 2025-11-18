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
