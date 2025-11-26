use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{error, info};

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
        limit: u64,
        offset: u64,
    ) -> Result<SearchEventServiceResponse, SearchEventServiceError> {
        let events = self
            .event_repository
            .find_between(start_time, end_time, limit, offset)
            .await
            .inspect_err(|e| {
                error!("Error searching events between {start_time:?} and {end_time:?}: {e:?}")
            })?;

        Ok(SearchEventServiceResponse {
            events,
            limit,
            offset,
        })
    }
}

pub struct SearchEventServiceResponse {
    pub events: Vec<Event>,
    pub limit: u64,
    pub offset: u64,
}

pub struct SearchEventServiceError;

impl From<anyhow::Error> for SearchEventServiceError {
    fn from(_value: anyhow::Error) -> Self {
        Self
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
            info!("Fetching event data from provider...");
            let provider_events = event_provider_client.fetch_events()
                .await
                .inspect_err(|error| {
                    error!(
                        "Error fetching event data from provider: {error:?}.\n\nEvent data ingestion failed.",
                    );
            })?;

            // 2. Insert or update events in repository depending on ingestion criteria
            info!(
                "Updating event store with provider data: {} entities to be processed",
                provider_events.len()
            );
            for pe in provider_events {
                // Ingestion skips individual entities when unexpected error happens in the repository
                let res = event_repository.find_by_title(&pe.title).await;
                if res.is_err() {
                    res.inspect_err(|error| {
                        error!("Error finding event by title in the event store: {error:?}")
                    })
                    .ok();
                    continue;
                }

                if let Some(mut e) = res.unwrap() {
                    // Upsert
                    e.start_time = pe.start_time;
                    e.end_time = pe.end_time;
                    e.min_price = pe.min_price;
                    e.max_price = pe.max_price;
                    event_repository
                        .upsert(e)
                        .await
                        .inspect_err(|error| {
                            error!("Error upserting event in event store: {error:?}")
                        })
                        .ok();
                } else {
                    // Save
                    event_repository
                        .save(pe.into())
                        .await
                        .inspect_err(|error| {
                            error!("Error saving new event in event store: {error:?}")
                        })
                        .ok();
                }
            }
            info!("Event store update finished.");

            Ok::<(), anyhow::Error>(())
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
