mod v1;

use std::sync::Arc;

use axum::Router;
use serde::Serialize;
use serde::ser::SerializeStruct;

use super::ApplicationState;

use crate::application::ports::provider::EventProviderClient;
use crate::application::ports::repository::EventRepository;

pub fn configure<T, S>(state: Arc<ApplicationState<T, S>>) -> Router
where
    T: EventRepository + Send + Sync + 'static,
    S: EventProviderClient + Send + Sync + 'static,
{
    Router::new().nest("/api/v1", v1::configure(state))
}

pub enum ApiResponse<T: Serialize, M: Serialize> {
    Ok(T, M),
    Ko(ErrorResponse),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl<T, M> Serialize for ApiResponse<T, M>
where
    T: serde::ser::Serialize,
    M: serde::ser::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut state = serializer.serialize_struct("ApiResponse", 2)?;

        match self {
            ApiResponse::Ok(data, meta) => {
                state.serialize_field("data", data)?;
                state.serialize_field("meta", meta)?;
                state.serialize_field("error", &Option::<ErrorResponse>::None)?;
            }
            ApiResponse::Ko(error) => {
                state.serialize_field("data", &Option::<T>::None)?;
                state.serialize_field("meta", &Option::<M>::None)?;
                state.serialize_field("error", error)?;
            }
        }

        state.end()
    }
}
