mod adapters;
mod config;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use log::{debug, info};
use sqlx::postgres::PgPoolOptions;
use tokio::{net::TcpListener, signal};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    application::service::{IngestEventService, SearchEventService},
    infrastructure::config::ApplicationConfig,
};

const DEFAULT_TRACING_ENV_FILTER: &'static str = "ferric_event_api=trace,tower_http=warn";
const APP_ENV_PREFIX: &'static str = "APP";
const APP_CONFIG_PREFIX_SEPARATOR: &'static str = "__";
const APP_CONFIG_SEPARATOR: &'static str = "__";
const DEFAULT_APP_PORT: u16 = 8080;

pub fn init_tracing() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new(DEFAULT_TRACING_ENV_FILTER))
                .context("Failed to initialize tracing")?,
        )
        .with(fmt::Layer::default())
        .init();

    Ok(())
}

pub fn load_config() -> anyhow::Result<config::ApplicationConfig> {
    let app_config = config::ApplicationConfig::new(
        APP_ENV_PREFIX,
        APP_CONFIG_PREFIX_SEPARATOR,
        APP_CONFIG_SEPARATOR,
    )
    .context("Failed to load application config")?;

    Ok(app_config)
}

pub fn serve_app(config: ApplicationConfig) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()?
        .block_on(async {
            let pool = PgPoolOptions::new()
                .max_connections(config.database.max_connections.clone())
                .connect(&config.database.url.clone())
                .await
                .context("Failed to create database connection pool")?;
            info!("Database connection pool established");
            let event_repository = adapters::repository::PostgresEventRepository::new(pool);

            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(
                    config.event_provider_client.request_timeout,
                ))
                .build()
                .context("Failed to create HTTP client for event provider client")?;
            let event_provider_client = adapters::provider::HttpEventProviderClient::new(
                config.event_provider_client.url.clone(),
                config.event_provider_client.api_path.clone(),
                client,
            );
            info!("Event Provider client initialized successfully");

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
            .await
            .context("Failed to initialize controller")?;

            // Server
            let port = config.port.unwrap_or(DEFAULT_APP_PORT);
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
            let listener = TcpListener::bind(addr).await?;
            info!("Server listening at {addr:?}");

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
            .expect("Failed to install Ctrl-C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending();

    tokio::select! {
        _ = ctrl_c => { debug!("SIGINT received") },
        _ = terminate => { debug!("SIGTERM received") },
    }
}
