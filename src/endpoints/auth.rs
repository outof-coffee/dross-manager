use std::sync::Arc;
use axum::{Extension, Json};
use axum::extract::Path;
use crate::prelude::*;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use crate::repository::RepositoryError;

pub async fn authenticate_user(
    Extension(player_repository): Extension<Arc<PlayerRepository>>,
    Path((email, token)): Path<(String, String)>) -> Response {

    let res = player_repository.login(email.clone(), token.clone()).await;

    // TODO: implement jwt
    match res {
        Ok(player) => {
            log::info!("Authenticated player {}", player.id);
            (StatusCode::OK, Json(player)).into_response()
        },
        Err(err) => {
            log::error!("Error authenticating player {}: {:?}", email, err);
            match err {
                RepositoryError::NotFound => {
                    (StatusCode::NOT_FOUND, Json("Not Found")).into_response()
                },
                _ => {
                    (StatusCode::INTERNAL_SERVER_ERROR, Json("Internal Server Error")).into_response()
                }
            }
        }
    }

}

pub async fn send_login_email(
    Extension(player_repository): Extension<Arc<PlayerRepository>>,
    Extension(email_repository): Extension<Arc<EmailRepository>>,
    Path(email): Path<String>) -> Response
{

    let player = player_repository.get_by_email(email.clone()).await;
    match player {
        Ok(player) => {
            log::info!("Sending login email to {}", email);
            let token = player_repository.generate_token(player.id).await;
            match token {
                Ok(token) => {
                    log::info!("Generated token for {}", email);
                    let res = email_repository.send_auth_token(email.clone().as_str(), token.as_str()).await;
                    match res {
                        Ok(_) => {
                            log::info!("Sent login email to {}", email);
                            (StatusCode::OK, Json("Sent")).into_response()
                        },
                        Err(err) => {
                            log::error!("Error sending login email to {}: {:?}", email, err);
                            (StatusCode::INTERNAL_SERVER_ERROR, Json("Internal Server Error")).into_response()
                        }
                    }
                },
                Err(err) => {
                    log::error!("Error generating token for {}: {:?}", email, err);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json("Internal Server Error")).into_response();
                }
            }
        },
        Err(err) => {
            log::error!("Could not find player with email address '{}': {:?}", email, err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json("Internal Server Error")).into_response()
        }
    }
}