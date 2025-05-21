use std::sync::Arc;

use axum::Router;
use axum::routing::{get, patch};

use crate::application::ports::provider::EventProviderClient;
use crate::application::ports::repository::EventRepository;
use crate::infrastructure::adapters::controller::handlers;
use crate::infrastructure::adapters::controller::state::ApplicationState;

pub fn configure<T, S>(state: Arc<ApplicationState<T, S>>) -> Router
where
    T: EventRepository + Send + Sync + 'static,
    S: EventProviderClient + Send + Sync + 'static,
{
    Router::new()
        .route("/", get(handlers::handle_root))
        .route("/search", get(handlers::handle_search))
        .route("/ingest", patch(handlers::handle_ingest))
        .with_state(state)
}
