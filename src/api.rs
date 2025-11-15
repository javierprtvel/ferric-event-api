use std::sync::Arc;

use axum::extract::State;
use axum::extract::{Query, rejection::QueryRejection};
use axum::http::StatusCode;
use axum::routing::{get, patch};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

use crate::state::ApplicationState;

pub fn configure(state: Arc<ApplicationState>) -> Router {
    Router::new()
        .route("/", get(handle_root))
        .route("/search", get(handle_search))
        .route("/ingest", patch(handle_ingest))
        .with_state(state)
}

async fn handle_root() -> Json<String> {
    Json("Hello, world!".to_string())
}

async fn handle_search(
    params: Result<Query<SearchParams>, QueryRejection>,
    State(state): State<Arc<ApplicationState>>,
) -> Result<Json<ApiResponse<SearchResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let ApplicationState {
        ref search_event_service,
        ..
    } = *state;

    match params {
        Err(_err) => {
            println!("Search query params are invalid");
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
            println!("Search query params are: {query:?}");

            let events = search_event_service
                .search_events(query.start_time, query.end_time)
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

            Ok(Json(ApiResponse::Ok(response)))
        }
    }
}

async fn handle_ingest(
    State(state): State<Arc<ApplicationState>>,
) -> Result<StatusCode, (StatusCode, Json<ApiResponse<()>>)> {
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
#[allow(dead_code)]
struct SearchParams {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

enum ApiResponse<T: Serialize> {
    Ok(T),
    Ko(ErrorResponse),
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

#[derive(Serialize)]
struct ErrorResponse {
    code: String,
    message: String,
}

#[derive(Serialize)]
struct SearchResponse {
    events: Vec<SearchEventResponse>,
}

#[derive(Serialize)]
struct SearchEventResponse {
    id: String,
    title: String,
    start_date: String,
    start_time: String,
    end_date: String,
    end_time: String,
    min_price: f64,
    max_price: f64,
}
