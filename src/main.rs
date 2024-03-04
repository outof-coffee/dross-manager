mod faery;
mod dross;
mod endpoints;
mod repository;
mod migrations;
mod version;
mod email;
mod player;
mod auth;
mod session;

use std::net::SocketAddr;
use axum::{routing::get, Router};
use tower_http::services::ServeDir;
use libsql::{Builder, Database};
use std::sync::Arc;
use axum::response::{IntoResponse};
use tokio::sync::Mutex;
use http::{Method};

use tower::{ServiceBuilder};
use tower_http::cors::{CorsLayer};


pub struct DrossManagerService {
    router: Router
}
pub struct DrossManagerState {
    pub player_repository: Arc<player::PlayerRepository>,
    pub faery_repository: Arc<faery::FaeryRepository>,
    pub email_repository: Arc<email::EmailRepository>,
    pub jwt_key_pair: JWTKeyPair
}

pub struct JWTKeyPair {
    pub public_key: String,
    pub private_key: String
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_secrets::Secrets] store: shuttle_secrets::SecretStore,
    // TODO: Remove entirely or replace with a real connection once shuttle_turso is fixed
    // #[shuttle_turso::Turso(
    //     addr = "",
    //     token = ""
    // )] turso: Connection
) -> Result<DrossManagerService, shuttle_runtime::Error> {

    let turso_addr = store.get("TURSO_URL").unwrap();
    let turso_token = store.get("TURSO_TOKEN").unwrap();
    let mailgun_user = store.get("MAILGUN_USER").unwrap();
    let mailgun_token = store.get("MAILGUN_PASSWORD").unwrap();
    let mailgun_domain = store.get("MAILGUN_DOMAIN").unwrap();
    let admin_email = store.get("ADMIN_EMAIL").unwrap();
    std::env::set_var("ADMIN_EMAIL", admin_email);
    let is_development = match std::env::var("ENVIRONMENT") {
        Ok(env) => env == "dev",
        _ => false
    };
    let db = if is_development {
        log::info!("using local path");
        Builder::new_local("dross_manager.sqlite").build().await.unwrap()
    } else {
        log::info!("using remote db");
        Builder::new_remote(turso_addr, turso_token).build().await.unwrap()
    };

    let db = Arc::new(Mutex::new(db));
    let state = Arc::new(DrossManagerState {
        player_repository: Arc::new(player::PlayerRepository::new(db.clone())),
        faery_repository: Arc::new(faery::FaeryRepository::new(db.clone())),
        email_repository: Arc::new(email::EmailRepository::new(mailgun_user, mailgun_token, mailgun_domain)),
        jwt_key_pair: JWTKeyPair {
            public_key: store.get("ACCESS_TOKEN_PUBLIC_KEY").unwrap(),
            private_key: store.get("ACCESS_TOKEN_PRIVATE_KEY").unwrap()
        }
    });

    // TODO: Handle errors
    let manager = migrations::Manager::new(db.clone(), state.player_repository.clone(), state.faery_repository.clone());
    let needs_migration = manager.needs_migration().await;
    if needs_migration {
        log::info!("Running migrations");
        manager.migrate().await.unwrap();
    }

    log::info!("Creating CORS middleware");
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]);

    log::info!("Creating router");
    let router = Router::new()
        .route("/api/hello", get(hello_world))
        .route("/api/faeries", get(endpoints::list_faeries).post(endpoints::create_faery))
        .route("/api/faeries/:faery_id", get(endpoints::get_faery).put(endpoints::update_faery).delete(endpoints::delete_faery))
        // .route("/api/test_email", get(send_test_email))
        .layer(ServiceBuilder::new().layer(cors))
        .with_state(state)
        .nest_service("/", ServeDir::new("dross-manager-frontend/dist"));

    Ok(DrossManagerService {
        router
    })
}

// async fn send_test_email(State(state): State<Arc<DrossManagerState>>) -> Response {
//     let res = state.clone().email_repository.send_email("Test", "tsalaroth@gmail.com", "This is a test email").await;
//     match res {
//         Ok(_) => (StatusCode::OK, Json("Sent")).into_response(),
//         Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
//     }
// }

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for DrossManagerService {
    async fn bind(mut self, addr: SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, self.router.clone()).await.unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::dross::{DrossHolder, transfer_dross};
    use crate::faery::Model;

    fn new_faery() -> Model {
        Model::new("Tinkerbell".to_string(), "me@example.com".to_string(), false, 0, None)
    }

    fn new_faery_two() -> Model {
        Model::new("Silvermist".to_string(), "you@example.com".to_string(), false, 0, None)
    }

    #[test]
    fn test_new_faery() {
        let faery = new_faery();
        assert_eq!(faery.name(), "Tinkerbell");
        assert_eq!(faery.email(), "me@example.com");
    }

    #[test]
    fn test_increment_dross() {
        let mut faery = new_faery();
        assert_eq!(faery.dross(), 0);
        let _ = faery.increment_dross(1);
        assert_eq!(faery.dross(), 1);
    }

    #[test]
    fn test_decrement_dross() {
        let mut faery = new_faery();
        assert_eq!(faery.dross(), 0);
        let _ = faery.increment_dross(1);
        assert_eq!(faery.dross(), 1);
        let _ = faery.decrement_dross(1);
        assert_eq!(faery.dross(), 0);
    }

    #[test]
    fn test_transfer_dross() {
        let mut faery = new_faery();
        let mut faery_two = new_faery_two();
        assert_eq!(faery.dross(), 0);
        assert_eq!(faery_two.dross(), 0);
        let _ = faery.increment_dross(1);
        assert_eq!(faery.dross(), 1);
        assert_eq!(faery_two.dross(), 0);
        transfer_dross(&mut faery, &mut faery_two, 1).unwrap();
        assert_eq!(faery.dross(), 0);
        assert_eq!(faery_two.dross(), 1);
    }
}