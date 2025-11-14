mod api;
mod provider;
mod repository;
mod service;
mod state;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

use chrono::DateTime;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::provider::EventProviderClient;
use crate::repository::{Event, EventRepository, SaveEventRequest};
use crate::service::{IngestEventService, SearchEventService};
use crate::state::ApplicationState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = init_application_state().await;

    let app = api::configure(state);

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn init_application_state() -> ApplicationState {
    let mut event_repository = EventRepository::new();
    event_repository
        .upsert(Event {
            id: Uuid::from_str("3fa85f64-5717-4562-b3fc-2c963f66afa6").unwrap(),
            title: "Quevedo".to_string(),
            start_time: DateTime::from_str("2025-11-12 22:00:00.000Z").unwrap(),
            end_time: DateTime::from_str("2025-11-12 23:00:00.000Z").unwrap(),
            min_price: 15.99f64,
            max_price: 39.99f64,
        })
        .await;
    event_repository
        .save(SaveEventRequest {
            title: "Nirvana".to_string(),
            start_time: DateTime::from_str("2025-10-31 16:30:00.000Z").unwrap(),
            end_time: DateTime::from_str("2025-10-31 23:59:59.000Z").unwrap(),
            min_price: 75.00f64,
            max_price: 99.99f64,
        })
        .await;
    event_repository
        .save(SaveEventRequest {
            title: "Tool".to_string(),
            start_time: DateTime::from_str("2025-12-24 21:00:00.000Z").unwrap(),
            end_time: DateTime::from_str("2025-12-24 23:45:00.000Z").unwrap(),
            min_price: 199.99f64,
            max_price: 199.99f64,
        })
        .await;

    let state = ApplicationState {
        search_event_service: SearchEventService::new(event_repository.clone()),
        ingest_event_service: IngestEventService::new(EventProviderClient, event_repository),
    };

    state
}
