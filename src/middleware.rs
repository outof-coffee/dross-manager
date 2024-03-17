pub mod auth {
    use std::sync::Arc;
    use axum::body::Body;
    use axum::extract::{Request, State};
    use axum::Json;
    use axum::middleware::Next;
    use axum::response::IntoResponse;
    use axum_extra::extract::CookieJar;
    use base64::Engine;
    use base64::engine::general_purpose;
    use http::{header, StatusCode};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use crate::DrossManagerState;
    use crate::prelude::PlayerData;
    use crate::repository::player::LoginResponse;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TokenDetails {
        pub token: Option<String>,
        pub token_uuid: Uuid,
        pub user_id: i64,
        pub expires_in: Option<i64>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TokenClaims {
        pub sub: String,
        pub token_uuid: String,
        pub exp: i64,
        pub iat: i64,
        pub nbf: i64,
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct JWTAuthMiddleware {
        pub user: PlayerData,
        pub access_token_uuid: Uuid,
    }

    #[derive(Debug, Serialize)]
    pub struct JWTErrorResponse {
        pub status: &'static str,
        pub message: String,
    }

    impl From<jsonwebtoken::errors::Error> for JWTErrorResponse {
        fn from(err: jsonwebtoken::errors::Error) -> Self {
            JWTErrorResponse {
                status: "error",
                message: format!("{:?}", err),
            }
        }
    }

    impl From<(LoginResponse, String)> for TokenDetails {
        fn from((login_response, private_key): (LoginResponse, String)) -> Self {
            // TODO: a real TTL from config somehow?
            match token_stub(login_response.id, 1_000_000, private_key) {
                Ok(token_details) => token_details,
                // TODO: maybe generate an "empty" token instead with an error optional?
                Err(_) => panic!("Failed to generate token"),
            }
        }
    }

    fn token_stub(user_id: i64, ttl: i64, private_key: String) -> Result<TokenDetails, JWTErrorResponse> {
        let bytes_private_key = general_purpose::STANDARD.decode(private_key).unwrap();
        let decoded_private_key = String::from_utf8(bytes_private_key).unwrap();
        let now = chrono::Utc::now();

        let mut token_details = TokenDetails {
            user_id,
            token_uuid: Uuid::new_v4(),
            expires_in: Some((now + chrono::Duration::try_minutes(ttl).unwrap()).timestamp()),
            token: None,
        };

        let claims = TokenClaims {
            sub: token_details.user_id.to_string(),
            token_uuid: token_details.token_uuid.to_string(),
            exp: token_details.expires_in.unwrap(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
        };

        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        let token = jsonwebtoken::encode(
            &header,
            &claims,
            &jsonwebtoken::EncodingKey::from_rsa_pem(decoded_private_key.as_bytes())?,
        )?;

        token_details.token = Some(token);
        Ok(token_details)
    }

// TODO: Clean up

    pub async fn example_authenticate(
        cookie_jar: CookieJar,
        State(app_state): State<Arc<DrossManagerState>>,
        req: Request<Body>,
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

        let access_token_details = match verify_jwt_token(app_state.jwt_key_pair.public_key.to_owned(), &access_token) {
            Ok(token_details) => token_details,
            Err(e) => {
                let error_response = JWTErrorResponse {
                    status: "fail",
                    message: format!("{:?}", e),
                };
                return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
            }
        };
        let _access_token_uuid = Uuid::parse_str(&access_token_details.token_uuid.to_string())
            .map_err(|_| {
                let error_response = JWTErrorResponse {
                    status: "fail",
                    message: "Invalid token".to_string(),
                };
                (StatusCode::UNAUTHORIZED, Json(error_response))
            })?;

        // let session =
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

    pub fn generate_jwt_token(user_id: i64, ttl: i64, private_key: String) -> Result<TokenDetails, JWTErrorResponse> {
        token_stub(user_id, ttl, private_key)
    }

    pub fn verify_jwt_token(
        public_key: String,
        token: &str,
    ) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
        let bytes_public_key = general_purpose::STANDARD.decode(public_key).unwrap();
        let decoded_public_key = String::from_utf8(bytes_public_key).unwrap();

        let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);

        let decoded = jsonwebtoken::decode::<TokenClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_rsa_pem(decoded_public_key.as_bytes())?,
            &validation,
        )?;

        let user_id: i64 = decoded.claims.sub.parse().unwrap();
        let token_uuid = Uuid::parse_str(decoded.claims.token_uuid.as_str()).unwrap();

        Ok(TokenDetails {
            token: None,
            token_uuid,
            user_id,
            expires_in: None,
        })
    }

}