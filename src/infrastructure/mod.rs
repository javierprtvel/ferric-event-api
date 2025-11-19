mod adapters;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use chrono::DateTime;
use tokio::{net::TcpListener, signal};
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
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

            // Controller
            let app =
                adapters::controller::init_controller(search_event_service, ingest_event_service)
                    .await?;

            // Tracing
            let subscriber = tracing_subscriber::registry()
                .with(LevelFilter::from_level(Level::TRACE))
                .with(fmt::Layer::default());
            subscriber.init();

            // Server
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
            let listener = TcpListener::bind(addr).await?;

            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await?;

            Ok::<(), anyhow::Error>(())
        })?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending();

    tokio::select! {
        _ = ctrl_c => {println!("SIGINT received")},
        _ = terminate => {println!("SIGTERM received")},
    }
}

#[allow(dead_code)]
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
