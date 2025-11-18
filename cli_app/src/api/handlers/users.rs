use crate::{
    api::{
        errors::AppError,
        response::users::{SingleUserResponse},
    },
    services::user::{UserService, CreateUserRequest, UpdateUserRequest},
    state::ApplicationState,
};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use std::sync::Arc;

pub async fn create(
    State(state): State<Arc<ApplicationState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<SingleUserResponse>, AppError> {
    let user = state.user_service.create_user(payload).await;

    match user {
        Err(e) => Err(AppError::from((StatusCode::NOT_FOUND, e))),
        Ok(user) => {
            let response = SingleUserResponse { data: user };
            Ok(Json(response))
        }
    }
}

pub async fn get(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<i64>,
) -> Result<Json<SingleUserResponse>, AppError> {
    let user = state.user_service.get_user_by_id(id).await;

    match user {
        Err(e) => Err(AppError::from((StatusCode::NOT_FOUND, e))),
        Ok(user) => {
            let response = SingleUserResponse { data: user };
            Ok(Json(response))
        }
    }
}

pub async fn update(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<SingleUserResponse>, AppError> {
    let user = state.user_service.update_user(id, payload).await;

    match user {
        Err(e) => Err(AppError::from((StatusCode::NOT_FOUND, e))),
        Ok(user) => {
            let response = SingleUserResponse { data: user };
            Ok(Json(response))
        }
    }
}

pub async fn delete(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    match state.user_service.delete_user(id).await {
        Err(e) => Err(AppError::from((StatusCode::NOT_FOUND, e))),
        Ok(_) => Ok(Json(())),
    }
}