use anyhow::Result;

use crate::application::ports::provider::{EventProviderClient, ProviderEvent};

#[allow(dead_code)]
pub struct DummyEventProviderClient;

impl EventProviderClient for DummyEventProviderClient {
    async fn fetch_events(&self) -> Result<Vec<ProviderEvent>> {
        Ok(Vec::new())
    }
}
