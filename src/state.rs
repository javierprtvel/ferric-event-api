use crate::service::{IngestEventService, SearchEventService};

pub struct ApplicationState {
    pub search_event_service: SearchEventService,
    pub ingest_event_service: IngestEventService,
}
