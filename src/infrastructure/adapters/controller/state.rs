use arc_swap::ArcSwap;

use crate::application::ports::{provider::EventProviderClient, repository::EventRepository};
use crate::application::service::{IngestEventService, SearchEventService};
use crate::infrastructure::config::ApplicationConfig;

pub struct ApplicationState<
    T: EventRepository + Send + Sync + 'static,
    S: EventProviderClient + Send + Sync + 'static,
> {
    pub config: ArcSwap<ApplicationConfig>,
    pub search_event_service: SearchEventService<T>,
    pub ingest_event_service: IngestEventService<S, T>,
}
