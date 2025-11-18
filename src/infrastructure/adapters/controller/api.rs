use std::sync::Arc;

use axum::Router;
use axum::routing::{get, patch};
use serde::Serialize;
use serde::ser::SerializeStruct;

use super::ApplicationState;
use super::handlers::*;

use crate::application::ports::provider::EventProviderClient;
use crate::application::ports::repository::EventRepository;

pub fn configure<T, S>(state: Arc<ApplicationState<T, S>>) -> Router
where
    T: EventRepository + Send + Sync + 'static,
    S: EventProviderClient + Send + Sync + 'static,
{
    Router::new()
        .route("/", get(handle_root))
        .route("/search", get(handle_search))
        .route("/ingest", patch(handle_ingest))
        .with_state(state)
}

pub enum ApiResponse<T: Serialize> {
    Ok(T),
    Ko(ErrorResponse),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl<T: serde::ser::Serialize> Serialize for ApiResponse<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut state = serializer.serialize_struct("ApiResponse", 2)?;

        match self {
            ApiResponse::Ok(data) => {
                state.serialize_field("data", data)?;
                state.serialize_field("error", &Option::<ErrorResponse>::None)?;
            }
            ApiResponse::Ko(error) => {
                state.serialize_field("data", &Option::<T>::None)?;
                state.serialize_field("error", error)?;
            }
        }

        state.end()
    }
}
