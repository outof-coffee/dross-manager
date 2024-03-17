mod dross;
mod version;
mod prelude;
mod repository;
mod endpoints;
mod middleware;

use std::net::SocketAddr;
use axum::{routing::get, Router, Extension};
use tower_http::services::ServeDir;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use http::{Method};
use prelude::*;

use tower::{ServiceBuilder};
use tower_http::cors::{CorsLayer};

pub struct DrossManagerService {
    router: Router
}
pub struct DrossManagerState {
    pub session_repository: Arc<SessionRepository>,
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
    #[shuttle_turso::Turso(
        addr = "{secrets.TURSO_URL}",
        token = "{secrets.TURSO_TOKEN}"
    )] turso: Connection
) -> Result<DrossManagerService, shuttle_runtime::Error> {

    let mailgun_user = store.get("MAILGUN_USER").unwrap();
    let mailgun_token = store.get("MAILGUN_PASSWORD").unwrap();
    let mailgun_domain = store.get("MAILGUN_DOMAIN").unwrap();
    let admin_email = store.get("ADMIN_EMAIL").unwrap();
    std::env::set_var("ADMIN_EMAIL", admin_email);

    let db = Arc::new(Mutex::new(turso));
    let faery_repository = Arc::new(FaeryRepository::new(db.clone()));
    let player_repository = Arc::new(PlayerRepository::new(db.clone()));
    let email_repository = Arc::new(EmailRepository::new(mailgun_user, mailgun_token, mailgun_domain));

    let state = Arc::new(DrossManagerState {
        // TODO: move to middleware manager
        session_repository: Arc::new(SessionRepository::new(db.clone())),
        jwt_key_pair: JWTKeyPair {
            public_key: store.get("ACCESS_TOKEN_PUBLIC_KEY").unwrap(),
            private_key: store.get("ACCESS_TOKEN_PRIVATE_KEY").unwrap()
        }
    });

    // TODO: Migrate to a migration service and / or remove the need to pass in the repositories like this
    let manager = migrations::Manager::new(db.clone(), player_repository.clone(), faery_repository.clone());
    log::info!("Running migrations");
    manager.migrate().await.unwrap();

    log::info!("Creating routers");
    log::info!("Creating faeries router");
    let faery_router = Router::new()
        .route("/", get(endpoints::faery::list_faeries).post(endpoints::faery::create_faery))
        .route("/:faery_id", get(endpoints::faery::get_faery)
            .put(endpoints::faery::update_faery)
            .delete(endpoints::faery::delete_faery))
        .layer(Extension(faery_repository));

    log::info!("Creating user router");
    let auth_router = Router::new()
        .route("/player/:email/:token", get(endpoints::auth::authenticate_user))
        .route("/player/:email", get(endpoints::auth::send_login_email))
        .layer(Extension(email_repository))
        .layer(Extension(player_repository));

    log::info!("Creating main api router");
    log::info!("Creating CORS middleware");
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]);
    let api_router = Router::new()
        .route("/hello", get(hello_world))
        .nest("/faeries", faery_router)
        .nest("/auth", auth_router)
        .layer(ServiceBuilder::new().layer(cors));

    log::info!("Creating web router");
    let web_router = Router::new()
        .nest_service("/", ServeDir::new("dross-manager-frontend/dist"));

    log::info!("Creating main application router");
    let router = Router::new()
        .nest("/api", api_router)
        .merge(web_router)
        .with_state(state);

    Ok(DrossManagerService {
        router
    })
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for DrossManagerService {
    async fn bind(mut self, addr: SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.router.clone()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::dross::{DrossHolder, transfer_dross};
    use crate::repository::faery::Model;

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