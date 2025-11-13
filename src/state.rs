use crate::service::SearchEventService;

#[derive(Clone)]
pub struct ApplicationState {
    pub search_event_service: SearchEventService,
}
