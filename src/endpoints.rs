use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::DrossManagerState;
use crate::repository::Repository;

pub async fn list_faeries(State(state): State<Arc<DrossManagerState>>) -> Response {
    log::info!("Getting all faeries");
    let res = state.clone().faery_repository.get_all().await;
    match res {
        Ok(res) => {
            log::info!("Got {} faeries", res.len());
            (StatusCode::OK, Json(res)).into_response()
        },
        Err(err) => {
            log::error!("Error getting all faeries: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn get_faery(State(state): State<Arc<DrossManagerState>>, Path(faery_id): Path<u32>) -> Response {
    log::info!("Getting faery {}", faery_id);
    let res = state.clone().faery_repository.get(faery_id).await;
    match res {
        Ok(res) => {
            log::info!("Got faery {}", faery_id);
            (StatusCode::OK, Json(res)).into_response()
        },
        Err(repo_err) => {
            log::error!("Error getting faery {}: {:?}", faery_id, repo_err);
            match repo_err {
                crate::repository::RepositoryError::NotFound => {
                    (StatusCode::NOT_FOUND, Json("Not Found")).into_response()
                },
                _ => {
                    (StatusCode::INTERNAL_SERVER_ERROR, Json("Internal Server Error")).into_response()
                }
            }
        }
    }
}