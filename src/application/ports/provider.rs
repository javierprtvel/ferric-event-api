use std::f64;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;

pub trait EventProviderClient {
    fn fetch_events(&self) -> impl std::future::Future<Output = Result<Vec<ProviderEvent>>> + Send;
}

#[derive(Debug, PartialEq)]
pub struct ProviderEvent {
    pub title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub min_price: f64,
    pub max_price: f64,
}
