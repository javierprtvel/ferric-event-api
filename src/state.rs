use crate::service::{IngestEventService, SearchEventService};

#[derive(Clone)]
pub struct ApplicationState {
    pub search_event_service: SearchEventService,
    pub ingest_event_service: IngestEventService,
}
