use std::f64;
use std::str::FromStr;

use anyhow::Result;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use serde::Deserialize;

#[derive(Clone)]
pub struct EventProviderClient;

const EVENT_PROVIDER_URL: &'static str = "https://localhost:8090";
const API_PATH: &'static str = "/api/events";

impl EventProviderClient {
    pub async fn fetch_events(&self) -> Result<Vec<ProviderEvent>> {
        let response_body_text = reqwest::get(format!("{EVENT_PROVIDER_URL}/{API_PATH}"))
            .await?
            .text()
            .await?;
        println!("Provider API response: {response_body_text}");

        let plan_list: EventPlanList = serde_xml_rs::from_str(&response_body_text)?;
        println!("Provider Plan list: {plan_list:?}");

        let events: Vec<ProviderEvent> = plan_list.into();
        println!("Provider events: {events:?}");

        Ok(events)
    }
}

#[derive(Debug)]
pub struct ProviderEvent {
    pub title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub min_price: f64,
    pub max_price: f64,
}

impl ProviderEvent {
    fn from(p: &Plan, title: &str) -> Result<Self> {
        let min_price = p
            .zones
            .iter()
            .map(|z| z.price.parse::<f64>().unwrap())
            .fold(f64::INFINITY, |a, b| a.min(b));
        let max_price = p
            .zones
            .iter()
            .map(|z| z.price.parse::<f64>().unwrap())
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));

        Ok(ProviderEvent {
            title: String::from(title),
            start_time: NaiveDateTime::from_str(&p.plan_start_date)?.and_utc(),
            end_time: NaiveDateTime::from_str(&p.plan_end_date)?.and_utc(),
            min_price,
            max_price,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename = "planList")]
struct EventPlanList {
    output: Output,
}

impl Into<Vec<ProviderEvent>> for EventPlanList {
    fn into(self) -> Vec<ProviderEvent> {
        self.output
            .base_plans
            .iter()
            .flat_map(|bp| {
                bp.plans
                    .iter()
                    .filter_map(|p| ProviderEvent::from(p, &bp.title).ok())
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct Output {
    #[serde(rename = "base_plan")]
    base_plans: Vec<BasePlan>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "base_plan")]
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
struct Zone {
    #[serde(rename = "@zone_id")]
    zone_id: String,
    #[serde(rename = "@price")]
    price: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_response_is_deserialized_and_mapped_correctly() {
        let response_text =
            std::fs::read_to_string("test/fixtures/provider_response_1.xml").unwrap();

        let plan_list: EventPlanList = serde_xml_rs::from_str(&response_text).unwrap();
        let events: Vec<ProviderEvent> = plan_list.into();

        assert_eq!(events.len(), 4);
    }
}
