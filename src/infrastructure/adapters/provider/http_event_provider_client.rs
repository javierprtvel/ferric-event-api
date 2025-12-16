use std::fmt::Debug;
use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use log::warn;
use reqwest::Client;
use reqwest::Url;
use serde::Deserialize;

use crate::application::ports::provider::{EventProviderClient, ProviderEvent};

pub struct HttpEventProviderClient {
    provider_url: String,
    event_api_path: String,
    client: reqwest::Client,
}

impl HttpEventProviderClient {
    pub fn new(provider_url: String, event_api_path: String, client: Client) -> Self {
        Self {
            provider_url,
            event_api_path,
            client,
        }
    }
}

impl EventProviderClient for HttpEventProviderClient {
    async fn fetch_events(&self) -> Result<Vec<ProviderEvent>> {
        let url = Url::parse(&format!("{}{}", self.provider_url, self.event_api_path))?;
        let response_body_text = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to fetch events from Event Provider API")?
            .text()
            .await
            .context("Failed to fetch events from Event Provider API")?;

        serde_xml_rs::from_str(&response_body_text)
            .map(EventPlanList::into)
            .context("Failed to fetch events from Event Provider API")
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename = "planList")]
struct EventPlanList {
    output: Output,
}

#[derive(Debug, Deserialize)]
struct Output {
    #[serde(rename = "base_plan")]
    base_plans: Vec<BasePlan>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "base_plan")]
#[allow(dead_code)]
struct BasePlan {
    #[serde(rename = "@base_plan_id")]
    base_plan_id: String,
    #[serde(rename = "@sell_mode")]
    sell_mode: String,
    #[serde(rename = "@title")]
    title: String,
    #[serde(rename = "plan")]
    plans: Vec<Plan>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Plan {
    #[serde(rename = "@plan_id")]
    plan_id: String,
    #[serde(rename = "@plan_start_date")]
    plan_start_date: String,
    #[serde(rename = "@plan_end_date")]
    plan_end_date: String,
    #[serde(rename = "zone")]
    zones: Vec<Zone>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Zone {
    #[serde(rename = "@zone_id")]
    zone_id: String,
    #[serde(rename = "@price")]
    price: String,
}

impl From<EventPlanList> for Vec<ProviderEvent> {
    fn from(value: EventPlanList) -> Self {
        value
            .output
            .base_plans
            .iter()
            .flat_map(|bp| {
                bp.plans.iter().filter_map(|p| {
                    ProviderEvent::from(p, &bp.title)
                        .inspect_err(|e| {
                            warn!("Failed to map event from Provider API to domain: {e:#}")
                        })
                        .ok()
                })
            })
            .collect()
    }
}

impl ProviderEvent {
    fn from(p: &Plan, title: &str) -> Result<Self> {
        let min_price = p
            .zones
            .iter()
            .filter_map(|z| z.price.parse::<f64>().ok())
            .fold(f64::INFINITY, |a, b| a.min(b));
        let max_price = p
            .zones
            .iter()
            .filter_map(|z| z.price.parse::<f64>().ok())
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));

        if min_price < f64::INFINITY || max_price > f64::NEG_INFINITY {
            Ok(ProviderEvent {
                title: String::from(title),
                start_time: NaiveDateTime::from_str(&p.plan_start_date)
                    .context(format!("Error parsing datetime {}", p.plan_start_date))?
                    .and_utc(),
                end_time: NaiveDateTime::from_str(&p.plan_end_date)
                    .context(format!("Error parsing datetime {}", p.plan_end_date))?
                    .and_utc(),
                min_price,
                max_price,
            })
        } else {
            Err(anyhow::Error::msg(
                "Error parsing zone prices: all prices are invalid",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::DateTime;
    use httpmock::MockServer;
    use reqwest::StatusCode;

    use super::*;

    #[test]
    fn provider_response_is_deserialized_and_mapped_correctly() {
        let response_text =
            std::fs::read_to_string("test/fixtures/provider_response_1.xml").unwrap();

        let plan_list: EventPlanList = serde_xml_rs::from_str(&response_text).unwrap();
        let provider_events: Vec<ProviderEvent> = plan_list.into();

        assert_eq!(provider_events.len(), 4);
        let expected = vec![
            ProviderEvent {
                title: "Camela en concierto".to_string(),
                start_time: DateTime::from_str("2021-06-30T21:00:00Z").unwrap(),
                end_time: DateTime::from_str("2021-06-30T22:00:00Z").unwrap(),
                min_price: 15.0f64,
                max_price: 30.0f64,
            },
            ProviderEvent {
                title: "Pantomima Full".to_string(),
                start_time: DateTime::from_str("2021-02-10T20:00:00Z").unwrap(),
                end_time: DateTime::from_str("2021-02-10T21:30:00Z").unwrap(),
                min_price: 55.0f64,
                max_price: 55.0f64,
            },
            ProviderEvent {
                title: "Pantomima Full".to_string(),
                start_time: DateTime::from_str("2021-02-11T20:00:00Z").unwrap(),
                end_time: DateTime::from_str("2021-02-11T21:30:00Z").unwrap(),
                min_price: 55.0f64,
                max_price: 55.0f64,
            },
            ProviderEvent {
                title: "Los Morancos".to_string(),
                start_time: DateTime::from_str("2021-07-31T20:00:00Z").unwrap(),
                end_time: DateTime::from_str("2021-07-31T21:00:00Z").unwrap(),
                min_price: 65.0f64,
                max_price: 75.0f64,
            },
        ];
        assert_eq!(provider_events, expected);
    }

    #[tokio::test]
    async fn client_fetches_events_from_provider_correctly() {
        let provider_server_mock = MockServer::start();
        let events_mock = provider_server_mock.mock(|when, then| {
            when.method("GET").path("/api/events");
            then.status(StatusCode::OK.as_u16())
                .header("content-type", "application/xml")
                .body(std::fs::read_to_string("test/fixtures/provider_response_2.xml").unwrap());
        });
        let client = HttpEventProviderClient::new(
            format!("http://localhost:{}", provider_server_mock.port()),
            "/api/events".to_string(),
            reqwest::Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap(),
        );

        let provider_events = client.fetch_events().await;

        events_mock.assert();
        assert!(provider_events.is_ok());
        let provider_events = provider_events.unwrap();
        assert_eq!(provider_events.len(), 3);
        let expected = vec![
            ProviderEvent {
                title: "El Clasico".to_string(),
                start_time: DateTime::from_str("2025-04-24T21:00:00Z").unwrap(),
                end_time: DateTime::from_str("2025-04-24T23:45:00Z").unwrap(),
                min_price: 120.0f64,
                max_price: 250.0f64,
            },
            ProviderEvent {
                title: "Bruce Springsteen toma Madrid".to_string(),
                start_time: DateTime::from_str("2025-08-31T18:00:00Z").unwrap(),
                end_time: DateTime::from_str("2025-08-31T22:30:00Z").unwrap(),
                min_price: 89.0f64,
                max_price: 199.99f64,
            },
            ProviderEvent {
                title: "Bruce Springsteen toma Madrid".to_string(),
                start_time: DateTime::from_str("2025-09-01T18:00:00Z").unwrap(),
                end_time: DateTime::from_str("2025-09-01T22:30:00Z").unwrap(),
                min_price: 75.95f64,
                max_price: 209.99f64,
            },
        ];
        assert_eq!(provider_events, expected);
    }

    #[tokio::test]
    async fn client_ignores_events_with_parsing_error_while_fetching_valid_ones() {
        let provider_server_mock = MockServer::start();
        let events_mock = provider_server_mock.mock(|when, then| {
            when.method("GET").path("/api/events");
            then.status(StatusCode::OK.as_u16())
                .header("content-type", "application/xml")
                .body(std::fs::read_to_string("test/fixtures/provider_response_3.xml").unwrap());
        });
        let client = HttpEventProviderClient::new(
            format!("http://localhost:{}", provider_server_mock.port()),
            "/api/events".to_string(),
            reqwest::Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap(),
        );

        let provider_events = client.fetch_events().await;

        events_mock.assert();
        assert!(provider_events.is_ok());
        let provider_events = provider_events.unwrap();
        assert_eq!(provider_events.len(), 2);
        let expected = vec![
            ProviderEvent {
                title: "Camela en concierto".to_string(),
                start_time: DateTime::from_str("2021-06-30T21:00:00Z").unwrap(),
                end_time: DateTime::from_str("2021-06-30T22:00:00Z").unwrap(),
                min_price: 15.0f64,
                max_price: 30.0f64,
            },
            ProviderEvent {
                title: "Los Morancos".to_string(),
                start_time: DateTime::from_str("2021-07-31T20:00:00Z").unwrap(),
                end_time: DateTime::from_str("2021-07-31T21:20:00Z").unwrap(),
                min_price: 65.0f64,
                max_price: 75.0f64,
            },
        ];
        assert_eq!(provider_events, expected);
    }
}
