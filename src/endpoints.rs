use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::DrossManagerState;
use crate::faery::Repository;

pub async fn list_faeries(State(state): State<Arc<DrossManagerState>>) -> Response {
    let res = state.clone().faery_repository.get_all().await;
    match res {
        Ok(res) => {
            (StatusCode::OK, Json(res)).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json("Internal Server Error")).into_response()
        }
    }
}

pub async fn get_faery(State(state): State<Arc<DrossManagerState>>, Path(faery_id): Path<u32>) -> Response {
    let res = state.clone().faery_repository.get(faery_id).await;
    match res {
        Ok(res) => {
            (StatusCode::OK, Json(res)).into_response()
        },
        Err(repo_err) => {
            match repo_err {
                crate::faery::RepositoryError::NotFound => {
                    (StatusCode::NOT_FOUND, Json("Not Found")).into_response()
                },
                _ => {
                    (StatusCode::INTERNAL_SERVER_ERROR, Json("Internal Server Error")).into_response()
                }
            }
        }
    }
}