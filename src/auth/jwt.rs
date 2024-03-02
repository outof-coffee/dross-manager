use std::sync::Arc;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::Json;
use axum::middleware::Next;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use axum_extra::extract::cookie::CookieJar;
use http::{header, StatusCode};
use crate::DrossManagerState;
use crate::player::PlayerData;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthMiddleware {
    pub user: PlayerData,
    pub access_token_uuid: uuid::Uuid,
}

#[derive(Debug, Serialize)]
pub struct JWTErrorResponse {
    pub status: &'static str,
    pub message: String,
}

pub async fn authenticate(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<DrossManagerState>>,
    mut req: Request<Body>,
    next: Next)  -> Result<impl IntoResponse, (StatusCode, Json<JWTErrorResponse>)> {
    let access_token = cookie_jar.get("access_token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    if auth_value.starts_with("Bearer ") {
                        Some(auth_value[7..].to_string())
                    } else {
                        None
                    }
                })
        });

    let access_token = access_token.ok_or_else(|| {
        let error_response = JWTErrorResponse {
            status: "fail",
            message: "You are not logged in, please provide token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;

    // let access_token_details =
    //     match token::verify_jwt_token(data.env.access_token_public_key.to_owned(), &access_token) {
    //         Ok(token_details) => token_details,
    //         Err(e) => {
    //             let error_response = ErrorResponse {
    //                 status: "fail",
    //                 message: format!("{:?}", e),
    //             };
    //             return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    //         }
    //     };
    // let access_token_uuid = uuid::Uuid::parse_str(&access_token_details.token_uuid.to_string())
    //     .map_err(|_| {
    //         let error_response = ErrorResponse {
    //             status: "fail",
    //             message: "Invalid token".to_string(),
    //         };
    //         (StatusCode::UNAUTHORIZED, Json(error_response))
    //     })?;
    //
    // let mut redis_client = data
    //     .redis_client
    //     .get_async_connection()
    //     .await
    //     .map_err(|e| {
    //         let error_response = ErrorResponse {
    //             status: "error",
    //             message: format!("Redis error: {}", e),
    //         };
    //         (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    //     })?;
    //
    // let redis_token_user_id = redis_client
    //     .get::<_, String>(access_token_uuid.clone().to_string())
    //     .await
    //     .map_err(|_| {
    //         let error_response = ErrorResponse {
    //             status: "error",
    //             message: "Token is invalid or session has expired".to_string(),
    //         };
    //         (StatusCode::UNAUTHORIZED, Json(error_response))
    //     })?;
    //
    // let user_id_uuid = uuid::Uuid::parse_str(&redis_token_user_id).map_err(|_| {
    //     let error_response = ErrorResponse {
    //         status: "fail",
    //         message: "Token is invalid or session has expired".to_string(),
    //     };
    //     (StatusCode::UNAUTHORIZED, Json(error_response))
    // })?;
    //
    // let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id_uuid)
    //     .fetch_optional(&data.db)
    //     .await
    //     .map_err(|e| {
    //         let error_response = ErrorResponse {
    //             status: "fail",
    //             message: format!("Error fetching user from database: {}", e),
    //         };
    //         (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    //     })?;
    //
    // let user = user.ok_or_else(|| {
    //     let error_response = ErrorResponse {
    //         status: "fail",
    //         message: "The user belonging to this token no longer exists".to_string(),
    //     };
    //     (StatusCode::UNAUTHORIZED, Json(error_response))
    // })?;
    //
    // req.extensions_mut().insert(JWTAuthMiddleware {
    //     user,
    //     access_token_uuid,
    // });
    Ok(next.run(req).await)

}
//
// fn generate_jwt_token(user_email: String, ttl: i64, private_key: String)