use crate::state::ApplicationState;

use axum::Router;
use std::sync::Arc;

mod errors;
mod handlers;
mod response;
mod v1;

pub fn configure(state: Arc<ApplicationState>) -> Router {
    Router::new().nest("/v1", v1::configure(state))
}
