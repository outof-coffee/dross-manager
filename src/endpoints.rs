use std::sync::Arc;
use axum::extract::{Path, State};
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::DrossManagerState;
use crate::repository::{Repository, RepositoryError};
use crate::repository::faery::{CreateFaeryRequest, Model};

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

pub async fn get_faery(State(state): State<Arc<DrossManagerState>>, Path(faery_id): Path<i64>) -> Response {
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

pub async fn update_faery(
    State(state): State<Arc<DrossManagerState>>,
    Path(faery_id): Path<i64>,
    payload: Result<Json<Model>, JsonRejection>
) -> Response {
    match payload {
        Ok(Json(payload)) => {
            if payload.id != Some(faery_id) {
                log::error!("Error updating faery {}: ID mismatch", faery_id);
                return (StatusCode::BAD_REQUEST, Json("ID mismatch")).into_response();
            }
            log::info!("Updating faery {}: {:?}", faery_id, payload);
            match state.clone().faery_repository.save(payload.clone()).await {
                Ok(_) => {
                    (StatusCode::OK, Json(payload)).into_response()
                },
                Err(err) => {
                    log::error!("Error updating faery {}: {:?}", faery_id, err);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
                }
            }
        },
        Err(err) => {
            log::error!("Error updating faery {}: {:?}", faery_id, err);
            let repo_error: RepositoryError = err.into();
            return (StatusCode::BAD_REQUEST, Json(repo_error)).into_response();
        }
    }
}

pub async fn create_faery(
    State(state): State<Arc<DrossManagerState>>,
    payload: Result<Json<CreateFaeryRequest>, JsonRejection>
) -> Response {
    match payload {
        Ok(Json(payload)) => {
            log::info!("Creating faery: {:?}", payload);
            let faery: Model = payload.into();
            match state.clone().faery_repository.create(Some(faery.clone())).await {
                Ok(_) => {
                    (StatusCode::CREATED, Json(faery)).into_response()
                },
                Err(err) => {
                    log::error!("Error creating faery: {:?}", err);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
                }
            }
        },
        Err(err) => {
            log::error!("Error creating faery: {:?}", err);
            let repo_error: RepositoryError = err.into();
            return (StatusCode::BAD_REQUEST, Json(repo_error)).into_response();
        }
    }

}

pub async fn delete_faery(State(state): State<Arc<DrossManagerState>>, Path(faery_id): Path<i64>) -> Response {
    log::info!("Deleting faery {}", faery_id);
    match state.clone().faery_repository.delete(faery_id).await {
        Ok(_) => {
            (StatusCode::NO_CONTENT, Json("")).into_response()
        },
        Err(err) => {
            log::error!("Error deleting faery {}: {:?}", faery_id, err);
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
    }
}