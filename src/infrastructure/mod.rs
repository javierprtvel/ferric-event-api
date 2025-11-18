mod adapters;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use chrono::DateTime;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::{
    application::{
        ports::repository::{EventRepository, SaveEventRequest},
        service::{IngestEventService, SearchEventService},
    },
    domain::event::Event,
};

pub fn serve_app() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()?
        .block_on(async {
            // let event_repository = adapters::repository::InMemoryEventRepository::new();
            // seed_event_repository(&event_repository).await;
            let event_repository = adapters::repository::DummyEventRepository;

            // let event_provider_client = adapters::provider::HttpEventProviderClient;
            let event_provider_client = adapters::provider::DummyEventProviderClient;

            // Dependency Injection
            let shared_event_repository = Arc::new(event_repository);
            let shared_event_provider_client = Arc::new(event_provider_client);

            let search_event_service = SearchEventService::new(shared_event_repository.clone());
            let ingest_event_service =
                IngestEventService::new(shared_event_provider_client, shared_event_repository);

            let app =
                adapters::controller::init_controller(search_event_service, ingest_event_service)
                    .await?;

            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
            let listener = TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;

            Ok::<(), anyhow::Error>(())
        })?;

    Ok(())
}

async fn seed_event_repository<T: EventRepository>(event_repository: &T) {
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
}
