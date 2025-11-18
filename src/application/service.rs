use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};

use super::ports::provider::{EventProviderClient, ProviderEvent};
use super::ports::repository::{EventRepository, SaveEventRequest};
use crate::domain::event::Event;

pub struct SearchEventService<T: EventRepository> {
    event_repository: Arc<T>,
}

impl<T: EventRepository> SearchEventService<T> {
    pub fn new(event_repository: Arc<T>) -> Self {
        Self { event_repository }
    }

    pub async fn search_events(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Vec<Event> {
        self.event_repository
            .find_between(start_time, end_time)
            .await
    }
}

pub struct IngestEventService<T: EventProviderClient, S: EventRepository> {
    event_provider_client: Arc<T>,
    event_repository: Arc<S>,
}

impl<T: EventProviderClient + Sync + Send + 'static, S: EventRepository + Sync + Send + 'static>
    IngestEventService<T, S>
{
    pub fn new(event_provider_client: Arc<T>, event_repository: Arc<S>) -> Self {
        Self {
            event_provider_client,
            event_repository,
        }
    }

    pub async fn ingest_events(&self) -> Result<()> {
        self.start_event_ingestion();
        Ok(())
    }

    fn start_event_ingestion(&self) {
        let event_provider_client = self.event_provider_client.clone();
        let event_repository = self.event_repository.clone();

        tokio::spawn(async move {
            // 1. Fetch event data from third-party event provider
            println!("Fetching event data from provider...");
            let provider_events = match event_provider_client.fetch_events().await {
                Ok(pes) => pes,
                Err(error) => {
                    println!(
                        "Error fetching event data from provider: {}.\nEvent data ingestion failed.",
                        error
                    );
                    return;
                }
            };

            // 2. Insert or update events in repository depending on ingestion criteria
            println!("Updating event store with provider data...");
            for pe in provider_events {
                if let Some(mut e) = event_repository.find_by_title(&pe.title).await {
                    // Upsert
                    e.start_time = pe.start_time;
                    e.end_time = pe.end_time;
                    e.min_price = pe.min_price;
                    e.max_price = pe.max_price;
                    event_repository.upsert(e).await;
                } else {
                    // Save
                    event_repository.save(pe.into()).await;
                }
            }
            println!("Event store update finished.");
        });
    }
}

impl Into<SaveEventRequest> for ProviderEvent {
    fn into(self) -> SaveEventRequest {
        SaveEventRequest {
            title: self.title,
            start_time: self.start_time,
            end_time: self.end_time,
            min_price: self.min_price,
            max_price: self.max_price,
        }
    }
}
