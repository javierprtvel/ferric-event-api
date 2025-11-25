use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::extract::{Query, rejection::QueryRejection};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::application::ports::provider::EventProviderClient;
use crate::application::ports::repository::EventRepository;

use super::api::{ApiResponse, ErrorResponse};
use super::state::ApplicationState;

pub async fn handle_root() -> Json<String> {
    Json("Hello, world!".to_string())
}

pub async fn handle_search<
    T: EventRepository + Send + Sync + 'static,
    S: EventProviderClient + Send + Sync + 'static,
>(
    params: Result<Query<SearchParams>, QueryRejection>,
    State(state): State<Arc<ApplicationState<T, S>>>,
) -> Result<
    Json<ApiResponse<SearchResponse, SearchMetadata>>,
    (StatusCode, Json<ApiResponse<(), ()>>),
> {
    let ApplicationState {
        ref search_event_service,
        ..
    } = *state;

    match params {
        Err(err) => {
            debug!("Search query params are invalid: {}", err);
            let error_response = ErrorResponse {
                code: "11".to_string(),
                message: "Missing required params".to_string(),
            };
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::Ko(error_response)),
            ))
        }
        Ok(query) => {
            debug!("Search query params are: {query:?}");

            let events = search_event_service
                .search_events(query.start_time, query.end_time, query.limit, query.offset)
                .await;

            let response = SearchResponse {
                events: events
                    .iter()
                    .map(|e| SearchEventResponse {
                        id: e.id.into(),
                        title: e.title.clone(),
                        start_date: e.start_time.format("%Y-%m-%d").to_string(),
                        start_time: e.start_time.format("%H:%M:%S").to_string(),
                        end_date: e.end_time.format("%Y-%m-%d").to_string(),
                        end_time: e.end_time.format("%H:%M:%S").to_string(),
                        min_price: e.min_price,
                        max_price: e.max_price,
                    })
                    .collect(),
            };

            Ok(Json(ApiResponse::Ok(
                response,
                SearchMetadata {
                    limit: query.limit,
                    offset: query.offset,
                },
            )))
        }
    }
}

pub async fn handle_ingest<
    T: EventRepository + Send + Sync + 'static,
    S: EventProviderClient + Send + Sync + 'static,
>(
    State(state): State<Arc<ApplicationState<T, S>>>,
) -> Result<StatusCode, (StatusCode, Json<ApiResponse<(), ()>>)> {
    let ApplicationState {
        ref ingest_event_service,
        ..
    } = *state;

    match ingest_event_service.ingest_events().await {
        Ok(()) => Ok(StatusCode::ACCEPTED),
        Err(_e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::Ko(ErrorResponse {
                code: "77".to_string(),
                message: "Unexpected error when starting event ingestion.".to_string(),
            })),
        )),
    }
}

#[derive(Deserialize, Debug)]
pub struct SearchParams {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    limit: u64,
    #[serde(default = "default_offset")]
    offset: u64,
}

fn default_offset() -> u64 {
    0
}

#[derive(Serialize)]
pub struct SearchResponse {
    events: Vec<SearchEventResponse>,
}

#[derive(Serialize)]
pub struct SearchEventResponse {
    id: String,
    title: String,
    start_date: String,
    start_time: String,
    end_date: String,
    end_time: String,
    min_price: f64,
    max_price: f64,
}

#[derive(Serialize)]
pub struct SearchMetadata {
    limit: u64,
    offset: u64,
}
