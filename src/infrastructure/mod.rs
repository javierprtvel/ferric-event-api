mod adapters;
mod config;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use sqlx::postgres::PgPoolOptions;
use tokio::{net::TcpListener, signal};
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    application::service::{IngestEventService, SearchEventService},
    infrastructure::config::ApplicationConfig,
};

pub fn load_config(env_prefix: &str) -> anyhow::Result<config::ApplicationConfig> {
    let app_config = config::ApplicationConfig::new(env_prefix)?;
    Ok(app_config)
}

pub fn serve_app(config: ApplicationConfig) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()?
        .block_on(async {
            let pool = PgPoolOptions::new()
                .max_connections(
                    config
                        .database
                        .max_connections
                        .clone()
                        .expect("Database maximum connections config value is missing"),
                )
                .connect(
                    &config
                        .database
                        .url
                        .clone()
                        .expect("Database URL config value is missing"),
                )
                .await?;
            let event_repository = adapters::repository::PostgresEventRepository::new(pool);

            let event_provider_client = adapters::provider::HttpEventProviderClient::new(
                config
                    .event_provider
                    .url
                    .clone()
                    .expect("Event provider URL config value is missing"),
                config
                    .event_provider
                    .api_path
                    .clone()
                    .expect("Event provider API path config value is missing"),
            );

            // Dependency Injection
            let shared_event_repository = Arc::new(event_repository);
            let shared_event_provider_client = Arc::new(event_provider_client);
            let search_event_service = SearchEventService::new(shared_event_repository.clone());
            let ingest_event_service =
                IngestEventService::new(shared_event_provider_client, shared_event_repository);

            // Controller
            let app = adapters::controller::init_controller(
                search_event_service,
                ingest_event_service,
                &config,
            )
            .await?;

            // Tracing
            let subscriber = tracing_subscriber::registry()
                .with(LevelFilter::from_level(Level::TRACE))
                .with(fmt::Layer::default());
            subscriber.init();

            // Server
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
            let listener = TcpListener::bind(addr).await?;
            println!("Server listening at {addr:?}");

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
