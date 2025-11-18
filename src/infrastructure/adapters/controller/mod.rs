mod api;
mod handlers;
mod state;

use std::sync::Arc;

use axum::Router;

use state::ApplicationState;

use crate::application::ports::provider::EventProviderClient;
use crate::application::ports::repository::EventRepository;
use crate::application::service::{IngestEventService, SearchEventService};

pub async fn init_controller<T, S>(
    search_event_service: SearchEventService<T>,
    ingest_event_service: IngestEventService<S, T>,
) -> anyhow::Result<Router>
where
    T: EventRepository + Send + Sync + 'static,
    S: EventProviderClient + Send + Sync + 'static,
{
    let state = init_application_state(search_event_service, ingest_event_service).await;
    let app = api::configure(Arc::new(state));
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
