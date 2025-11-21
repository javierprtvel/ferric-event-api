mod adapters;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use sqlx::postgres::PgPoolOptions;
use tokio::{net::TcpListener, signal};
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::application::service::{IngestEventService, SearchEventService};

const DATABASE_CONN_POOL_MAX_CONN: u32 = 30;
const DATABASE_URL: &'static str = "postgres://user:password@localhost:5432/eventdb";

pub fn serve_app() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()?
        .block_on(async {
            let pool = PgPoolOptions::new()
                .max_connections(DATABASE_CONN_POOL_MAX_CONN)
                .connect(DATABASE_URL)
                .await?;
            let event_repository = adapters::repository::PostgresEventRepository::new(pool);

            let event_provider_client = adapters::provider::HttpEventProviderClient;

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
