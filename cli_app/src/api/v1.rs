use super::handlers;
use crate::state::ApplicationState;

use axum::Router;
use axum::routing::{delete, get, post, put};
use std::sync::Arc;

pub fn configure(state: Arc<ApplicationState>) -> Router {
    Router::new()
        .route("/hello", get(handlers::hello::hello))
        .route("/posts", post(handlers::posts::create))
        .route("/posts", get(handlers::posts::list))
        .route("/posts/{id}", get(handlers::posts::get))
        .route("/posts/{id}", put(handlers::posts::update))
        .route("/posts/{id}", delete(handlers::posts::delete))
        .route("/users", post(handlers::users::create))
        .route("/users/{id}", get(handlers::users::get))
        .route("/users/{id}", put(handlers::users::update))
        .route("/users/{id}", delete(handlers::users::delete))
        .with_state(state)
}
