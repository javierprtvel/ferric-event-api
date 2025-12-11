mod api;
mod handlers;
mod state;

use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use axum::Router;

use reqwest::StatusCode;
use state::ApplicationState;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use crate::application::ports::provider::EventProviderClient;
use crate::application::ports::repository::EventRepository;
use crate::application::service::{IngestEventService, SearchEventService};
use crate::infrastructure::config::ApplicationConfig;

pub async fn init_controller<T, S>(
    search_event_service: SearchEventService<T>,
    ingest_event_service: IngestEventService<S, T>,
    config: &ApplicationConfig,
) -> anyhow::Result<Router>
where
    T: EventRepository + Send + Sync + 'static,
    S: EventProviderClient + Send + Sync + 'static,
{
    let state = init_application_state(config, search_event_service, ingest_event_service).await;

    let app = api::configure(Arc::new(state))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(config.api.request_timeout_secs),
        ))
        .layer(TraceLayer::new_for_http());

    Ok(app)
}

async fn init_application_state<T, S>(
    config: &ApplicationConfig,
    search_event_service: SearchEventService<T>,
    ingest_event_service: IngestEventService<S, T>,
) -> ApplicationState<T, S>
where
    T: EventRepository + Send + Sync + 'static,
    S: EventProviderClient + Send + Sync + 'static,
{
    ApplicationState {
        config: ArcSwap::new(Arc::new(config.clone())),
        search_event_service,
        ingest_event_service,
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::DateTime;
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::*;
    use crate::{
        domain::event::Event,
        infrastructure::adapters::{
            provider::DummyEventProviderClient,
            repository::{DummyEventRepository, FailingEventRepository},
        },
    };

    #[tokio::test]
    async fn root_endpoint_returns_ok_response() {
        let event_repository = Arc::new(DummyEventRepository(HashMap::new()));
        let event_provider_client = Arc::new(DummyEventProviderClient);
        let search_event_service = SearchEventService::new(event_repository.clone());
        let ingest_event_service = IngestEventService::new(event_provider_client, event_repository);
        let config = ApplicationConfig::default();
        let app = init_controller(search_event_service, ingest_event_service, &config)
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Value =
            serde_json::from_slice(&(response.into_body().collect().await.unwrap().to_bytes()))
                .unwrap();
        assert_eq!(
            body,
            json!({ "data": {"greetings": "Hello, world!"}, "error": null, "meta": null })
        );
    }

    #[tokio::test]
    async fn event_search_endpoint_returns_events_within_datetimes() {
        let event_repository = Arc::new(DummyEventRepository(HashMap::from([(
            Uuid::from_str("3fa85f64-5717-4562-b3fc-2c963f66afa6").unwrap(),
            Event {
                id: Uuid::from_str("3fa85f64-5717-4562-b3fc-2c963f66afa6").unwrap(),
                title: "Quevedo".to_string(),
                start_time: DateTime::from_str("2025-11-12T22:00:00Z").unwrap(),
                end_time: DateTime::from_str("2025-11-12T23:00:00Z").unwrap(),
                min_price: 15.99,
                max_price: 39.99,
            },
        )])));
        let event_provider_client = Arc::new(DummyEventProviderClient);
        let search_event_service = SearchEventService::new(event_repository.clone());
        let ingest_event_service = IngestEventService::new(event_provider_client, event_repository);
        let config = ApplicationConfig::default();
        let app = init_controller(search_event_service, ingest_event_service, &config)
            .await
            .unwrap();

        let response = app.oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/search?start_time=2025-11-01T08:00:00Z&end_time=2025-11-30T18:00:00Z&limit=100")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Value =
            serde_json::from_slice(&(response.into_body().collect().await.unwrap().to_bytes()))
                .unwrap();
        assert_eq!(
            body,
            json!({
                "data": {
                    "events": [{
                        "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
                        "title": "Quevedo",
                        "start_date": "2025-11-12",
                        "start_time": "22:00:00",
                        "end_date": "2025-11-12",
                        "end_time": "23:00:00",
                        "min_price": 15.99,
                        "max_price": 39.99,
                    }]
                },
                "error": null,
                "meta": {
                    "limit": 100,
                    "offset": 0,
                }
            })
        )
    }

    #[tokio::test]
    async fn event_search_endpoint_returns_client_error_when_required_param_is_missing() {
        let event_repository = Arc::new(DummyEventRepository(HashMap::new()));
        let event_provider_client = Arc::new(DummyEventProviderClient);
        let search_event_service = SearchEventService::new(event_repository.clone());
        let ingest_event_service = IngestEventService::new(event_provider_client, event_repository);
        let config = ApplicationConfig::default();
        let app = init_controller(search_event_service, ingest_event_service, &config)
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/search?start_time=2025-11-01T08:00:00Z&limit=100")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn event_search_endpoint_returns_server_error_when_something_unexpected_happens() {
        let event_repository = Arc::new(FailingEventRepository);
        let event_provider_client = Arc::new(DummyEventProviderClient);
        let search_event_service = SearchEventService::new(event_repository.clone());
        let ingest_event_service = IngestEventService::new(event_provider_client, event_repository);
        let config = ApplicationConfig::default();
        let app = init_controller(search_event_service, ingest_event_service, &config)
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/search?start_time=2025-11-01T08:00:00Z&end_time=2025-11-30T18:00:00Z&limit=100")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body: Value =
            serde_json::from_slice(&(response.into_body().collect().await.unwrap().to_bytes()))
                .unwrap();
        assert_eq!(
            body,
            json!({ "data": null, "meta": null, "error": { "code": "99", "message": "Unexpected error when searching events." } })
        )
    }

    #[tokio::test]
    async fn event_ingest_endpoint_returns_accepted() {
        let event_repository = Arc::new(DummyEventRepository(HashMap::new()));
        let event_provider_client = Arc::new(DummyEventProviderClient);
        let search_event_service = SearchEventService::new(event_repository.clone());
        let ingest_event_service = IngestEventService::new(event_provider_client, event_repository);
        let config = ApplicationConfig::default();
        let app = init_controller(search_event_service, ingest_event_service, &config)
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/v1/ingest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }
}
