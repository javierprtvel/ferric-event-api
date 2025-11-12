use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::extract::{Query, rejection::QueryRejection};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = Router::new()
        .route("/", get(async || Json("Hello, world!")))
        .route("/search", get(handle_search));

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, router).await?;

    Ok(())
}

async fn handle_search(
    params: Result<Query<SearchParams>, QueryRejection>,
) -> Result<Json<ApiResponse<SearchResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match params {
        Err(_err) => {
            println!("Search query params are invalid");
            let error_response = ErrorResponse {
                code: "11".to_string(),
                message: "Missing required params".to_string(),
            };
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::Ko(error_response)),
            ))
        }
        Ok(query) => {
            println!("Search query params are: {query:?}");
            let dummy_response = SearchResponse {
                events: vec![SearchEventResponse {
                    id: "3fa85f64-5717-4562-b3fc-2c963f66afa6".to_string(),
                    title: "Quevedo".to_string(),
                    start_date: "2025-11-12".to_string(),
                    start_time: "22:38:19".to_string(),
                    end_date: "2025-11-12".to_string(),
                    end_time: "14:45:15".to_string(),
                    min_price: 15.99f64,
                    max_price: 39.99f64,
                }],
            };

            Ok(Json(ApiResponse::Ok(dummy_response)))
        }
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
