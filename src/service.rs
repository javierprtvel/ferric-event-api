use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::{
    provider::{EventProviderClient, ProviderEvent},
    repository::{Event, EventRepository, SaveEventRequest},
};

#[derive(Clone)]
pub struct SearchEventService {
    event_repository: EventRepository,
}

impl SearchEventService {
    pub fn new(event_repository: EventRepository) -> Self {
        Self { event_repository }
    }

    pub async fn search_events(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Vec<&Event> {
        self.event_repository
            .find_between(start_time, end_time)
            .await
    }
}

#[derive(Clone)]
pub struct IngestEventService {
    event_provider_client: EventProviderClient,
    event_repository: EventRepository,
}

impl IngestEventService {
    pub fn new(
        event_provider_client: EventProviderClient,
        event_repository: EventRepository,
    ) -> Self {
        Self {
            event_provider_client,
            event_repository,
        }
    }

    pub async fn ingest_events(&mut self) -> Result<()> {
        // 1. Fetch event data from third-party provider
        let provider_events = self.event_provider_client.fetch_events().await?;

        // 2. Insert or update events in repository depending on ingestion criteria
        for pe in provider_events {
            if let Some(mut e) = self.event_repository.find_by_title(&pe.title).await {
                // Upsert
                e.start_time = pe.start_time;
                e.end_time = pe.end_time;
                e.min_price = pe.min_price;
                e.max_price = pe.max_price;
                self.event_repository.upsert(e).await;
            } else {
                // Save
                self.event_repository.save(pe.into()).await;
            }
        }

        Ok(())
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
