mod api;
mod handlers;
mod state;

use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use axum::Router;

use reqwest::StatusCode;
use state::ApplicationState;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use crate::application::ports::provider::EventProviderClient;
use crate::application::ports::repository::EventRepository;
use crate::application::service::{IngestEventService, SearchEventService};
use crate::infrastructure::config::ApplicationConfig;

pub async fn init_controller<T, S>(
    search_event_service: SearchEventService<T>,
    ingest_event_service: IngestEventService<S, T>,
    config: &ApplicationConfig,
) -> anyhow::Result<Router>
where
    T: EventRepository + Send + Sync + 'static,
    S: EventProviderClient + Send + Sync + 'static,
{
    let state = init_application_state(&config, search_event_service, ingest_event_service).await;

    let app = api::configure(Arc::new(state))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(config.api.request_timeout_secs),
        ))
        .layer(TraceLayer::new_for_http());

    Ok(app)
}

async fn init_application_state<T, S>(
    config: &ApplicationConfig,
    search_event_service: SearchEventService<T>,
    ingest_event_service: IngestEventService<S, T>,
) -> ApplicationState<T, S>
where
    T: EventRepository + Send + Sync + 'static,
    S: EventProviderClient + Send + Sync + 'static,
{
    ApplicationState {
        config: ArcSwap::new(Arc::new(config.clone())),
        search_event_service,
        ingest_event_service,
    }
}
