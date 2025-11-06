use crate::{
    api::{
        errors::AppError,
        response::posts::{ListPostsResponse, SinglePostResponse},
    },
    services::post::{CreatePostRequest, PostService, UpdatePostRequest},
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
    Json(payload): Json<CreatePostRequest>,
) -> Result<Json<SinglePostResponse>, AppError> {
    let post = state.post_service.create_post(payload).await;

    match post {
        Err(e) => Err(AppError::from((StatusCode::NOT_FOUND, e))),
        Ok(post) => {
            let response = SinglePostResponse { data: post };
            Ok(Json(response))
        }
    }
}

pub async fn list(
    State(state): State<Arc<ApplicationState>>,
) -> Result<Json<ListPostsResponse>, AppError> {
    let posts = state.post_service.get_all_posts().await?;

    let response = ListPostsResponse { data: posts };

    Ok(Json(response))
}

pub async fn get(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<i64>,
) -> Result<Json<SinglePostResponse>, AppError> {
    let post = state.post_service.get_post_by_id(id).await;

    match post {
        Err(e) => Err(AppError::from((StatusCode::NOT_FOUND, e))),
        Ok(post) => {
            let response = SinglePostResponse { data: post };
            Ok(Json(response))
        }
    }
}

pub async fn update(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<Json<SinglePostResponse>, AppError> {
    let post = state.post_service.update_post(id, payload).await;

    match post {
        Err(e) => Err(AppError::from((StatusCode::NOT_FOUND, e))),
        Ok(post) => {
            let response = SinglePostResponse { data: post };
            Ok(Json(response))
        }
    }
}

pub async fn delete(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    match state.post_service.delete_post(id).await {
        Err(e) => Err(AppError::from((StatusCode::NOT_FOUND, e))),
        Ok(_) => Ok(Json(())),
    }
}
