use chrono::{DateTime, Utc};

use crate::repository::{Event, EventRepository};

#[derive(Clone)]
pub struct SearchEventService {
    event_repository: EventRepository,
}

impl SearchEventService {
    pub fn new(event_repository: EventRepository) -> Self {
        Self { event_repository }
    }

    pub fn search_events(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Vec<&Event> {
        self.event_repository.find_between(start_time, end_time)
    }
}
