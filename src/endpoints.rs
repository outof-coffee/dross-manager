use std::sync::Arc;
use axum::extract::{Path, State};
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use crate::DrossManagerState;
use crate::faery::Faery;
use crate::repository::{Repository, RepositoryError};

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

pub async fn update_faery(
    State(state): State<Arc<DrossManagerState>>,
    Path(faery_id): Path<u32>,
    payload: Result<Json<Faery>, JsonRejection>
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
            return (StatusCode::BAD_REQUEST, Json(RepositoryError::from(err))).into_response();
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct CreateFaeryRequest {
    pub name: String,
    pub email: String,
}
pub async fn create_faery(
    State(state): State<Arc<DrossManagerState>>,
    payload: Result<Json<CreateFaeryRequest>, JsonRejection>
) -> Response {
    match payload {
        Ok(Json(payload)) => {
            log::info!("Creating faery: {:?}", payload);
            let faery = Faery::new(payload.name, payload.email, false, 0, None);
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
            return (StatusCode::BAD_REQUEST, Json(RepositoryError::from(err))).into_response();
        }
    }

}