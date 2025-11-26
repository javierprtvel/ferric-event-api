use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::ports::repository::{EventRepository, SaveEventRequest};
use crate::domain::event::Event;

pub struct PostgresEventRepository(PgPool);

impl PostgresEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
}

impl EventRepository for PostgresEventRepository {
    async fn find_all(&self) -> Result<Vec<Event>> {
        let query = sqlx::query_as!(
            PostgresEvent,
            r#"
            SELECT id, title, start_time, end_time, min_price as min_price_in_lowest_denomination, max_price as max_price_in_lowest_denomination
            FROM events
        "#
        );

        query
            .fetch_all(&self.0)
            .await
            .context("Failed to find all events in event database")
            .map(postgres_events_into_domain_events)
    }

    async fn find_between(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Event>> {
        let signed_limit: i64 = limit
            .try_into()
            .context("Failed to cast query limit from u64 to i64")?;
        let signed_offset: i64 = offset
            .try_into()
            .context("Failed to cast query offset from u64 to i64")?;

        let query = sqlx::query_as!(
            PostgresEvent,
            r#"
            SELECT id, title, start_time, end_time, min_price as min_price_in_lowest_denomination, max_price as max_price_in_lowest_denomination
            FROM events
            WHERE start_time >= $1 AND end_time <= $2
            LIMIT $3
            OFFSET $4
        "#,
            start_time,
            end_time,
            signed_limit,
            signed_offset,
        );

        query
            .fetch_all(&self.0)
            .await
            .context("Failed to find events between datetimes in event database")
            .map(postgres_events_into_domain_events)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Event>> {
        let query = sqlx::query_as!(
            PostgresEvent,
            r#"
                SELECT id, title, start_time, end_time, min_price as min_price_in_lowest_denomination, max_price as max_price_in_lowest_denomination
                FROM events
                WHERE id = $1
            "#,
            id,
        );

        query
            .fetch_optional(&self.0)
            .await
            .context("Failed to find event by id in event database")
            .map(|optional| optional.map(PostgresEvent::into))
    }

    async fn find_by_title(&self, title: &str) -> Result<Option<Event>> {
        let query = sqlx::query_as!(
            PostgresEvent,
            r#"
            SELECT id, title, start_time, end_time, min_price as min_price_in_lowest_denomination, max_price as max_price_in_lowest_denomination
            FROM events
            WHERE title = $1
        "#,
            title
        );

        query
            .fetch_optional(&self.0)
            .await
            .context("Failed to find event by title in event database")
            .map(|optional| optional.map(PostgresEvent::into))
    }

    async fn save(&self, e: SaveEventRequest) -> Result<Event> {
        let event = PostgresEvent::from(e);
        let query = sqlx::query!(
            r#"
                INSERT INTO events (id, title, start_time, end_time, min_price, max_price)
                VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            event.id,
            event.title,
            event.start_time,
            event.end_time,
            event.min_price_in_lowest_denomination,
            event.max_price_in_lowest_denomination,
        );

        query
            .execute(&self.0)
            .await
            .context("Failed to insert event record in event database")?;

        self.find_by_id(&event.id)
            .await
            .context("Failed to save event in event database")?
            .ok_or(anyhow!("Could not find saved entity by its id"))
            .context("Failed to save event in event database")
    }

    async fn upsert(&self, entity: Event) -> Result<Event> {
        let event = PostgresEvent::from(entity);
        let query = sqlx::query!(
            r#"
                INSERT INTO events (id, title, start_time, end_time, min_price, max_price)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (id) DO UPDATE
                SET title = $2, start_time = $3, end_time = $4, min_price = $5, max_price = $6
            "#,
            event.id,
            event.title,
            event.start_time,
            event.end_time,
            event.min_price_in_lowest_denomination,
            event.max_price_in_lowest_denomination,
        );

        query
            .execute(&self.0)
            .await
            .context("Failed to update event record in event database")?;

        self.find_by_id(&event.id)
            .await
            .context("Failed to upsert event in event database")?
            .ok_or(anyhow!("Could not find upserted entity by its id"))
            .context("Failed to upsert event in event database")
    }
}

#[derive(sqlx::FromRow, Clone)]
struct PostgresEvent {
    id: Uuid,
    title: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    min_price_in_lowest_denomination: i32,
    max_price_in_lowest_denomination: i32,
}

impl Into<Event> for PostgresEvent {
    fn into(self) -> Event {
        Event {
            id: self.id,
            title: self.title,
            start_time: self.start_time,
            end_time: self.end_time,
            min_price: self.min_price_in_lowest_denomination as f64 / 100.0,
            max_price: self.max_price_in_lowest_denomination as f64 / 100.0,
        }
    }
}

impl From<Event> for PostgresEvent {
    fn from(value: Event) -> Self {
        Self {
            id: value.id,
            title: value.title,
            start_time: value.start_time,
            end_time: value.end_time,
            min_price_in_lowest_denomination: (value.min_price * 100.0) as i32,
            max_price_in_lowest_denomination: (value.max_price * 100.0) as i32,
        }
    }
}

impl From<SaveEventRequest> for PostgresEvent {
    fn from(value: SaveEventRequest) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: value.title,
            start_time: value.start_time,
            end_time: value.end_time,
            min_price_in_lowest_denomination: (value.min_price * 100.0) as i32,
            max_price_in_lowest_denomination: (value.max_price * 100.0) as i32,
        }
    }
}

fn postgres_events_into_domain_events(values: Vec<PostgresEvent>) -> Vec<Event> {
    values.iter().map(|pe| pe.clone().into()).collect()
}
