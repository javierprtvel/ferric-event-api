use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;

pub struct AppError(StatusCode, anyhow::Error);

#[derive(Serialize)]
struct ErrorResponse {
    detail: String,
}

impl From<(StatusCode, anyhow::Error)> for AppError {
    fn from((status_code, value): (StatusCode, anyhow::Error)) -> Self {
        Self(status_code, value)
    }
}

// This allows ? to automatically convert anyhow::Error to AppError
impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let detail = format!("{:?}", self.1);
        (self.0, Json(ErrorResponse { detail })).into_response()
    }
}
