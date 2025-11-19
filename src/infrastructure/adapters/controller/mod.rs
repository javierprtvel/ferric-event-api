mod api;
mod handlers;
mod state;

use std::sync::Arc;
use std::time::Duration;

use axum::Router;

use state::ApplicationState;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use crate::application::ports::provider::EventProviderClient;
use crate::application::ports::repository::EventRepository;
use crate::application::service::{IngestEventService, SearchEventService};

const REQUEST_TIMEOUT_SECS: u64 = 30;

pub async fn init_controller<T, S>(
    search_event_service: SearchEventService<T>,
    ingest_event_service: IngestEventService<S, T>,
) -> anyhow::Result<Router>
where
    T: EventRepository + Send + Sync + 'static,
    S: EventProviderClient + Send + Sync + 'static,
{
    let state = init_application_state(search_event_service, ingest_event_service).await;
    let app = api::configure(Arc::new(state))
        .layer(TimeoutLayer::new(Duration::from_secs(REQUEST_TIMEOUT_SECS)))
        .layer(TraceLayer::new_for_http());
    Ok(app)
}

async fn init_application_state<T, S>(
    search_event_service: SearchEventService<T>,
    ingest_event_service: IngestEventService<S, T>,
) -> ApplicationState<T, S>
where
    T: EventRepository + Send + Sync + 'static,
    S: EventProviderClient + Send + Sync + 'static,
{
    ApplicationState {
        search_event_service,
        ingest_event_service,
    }
}
