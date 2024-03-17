use std::sync::Arc;
use libsql::Connection;
use axum::{Extension, Router};
use axum::routing::get;
use tower_http::cors::CorsLayer;
use http::Method;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_sessions::SessionManagerLayer;
use tower_sessions_core::Expiry;
use time::Duration;
use tokio::sync::Mutex;
use crate::{endpoints, middleware};
use crate::prelude::{EmailRepository, FaeryRepository, migrations, PlayerRepository};

pub struct DrossManagerService {
    router: Router
}

pub struct DrossManagerState {
    pub jwt_key_pair: JWTKeyPair
}

pub struct JWTKeyPair {
    pub public_key: String,
    pub private_key: String
}

pub async fn create_router_tree(
    faery_repository: Arc<FaeryRepository>,
    email_repository: Arc<EmailRepository>,
    player_repository: Arc<PlayerRepository>,
    state: Arc<DrossManagerState>,
    turso: Connection
) -> Router {
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
        .route("/hello", get(endpoints::hello_world))
        .nest("/faeries", faery_router)
        .nest("/auth", auth_router)
        .layer(ServiceBuilder::new().layer(cors));

    log::info!("Creating web router");
    let web_router = Router::new()
        .nest_service("/", ServeDir::new("dross-manager-frontend/dist"));

    log::info!("Creating Session middleware");
    log::info!("Creating Session Store");
    let session_store = middleware::session::LibSqlStore::new(turso.clone());
    log::info!("Creating session table");
    session_store.migrate().await.unwrap();

    // TODO: implement task select
    // log::info!("Creating expiration task");
    // let deletion_task = tokio::task::spawn(
    //     session_store
    //         .clone()
    //         .continuously_delete_expired(tokio::time::Duration::from_secs(5)),
    // );

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(60 * 60 * 24 * 7)));

    log::info!("Creating main application router");
    let router = Router::new()
        .nest("/api", api_router)
        .merge(web_router)
        .layer(session_layer)
        .with_state(state);
    router
}

pub fn setup_environment(store: &shuttle_secrets::SecretStore) -> (String, String, String, JWTKeyPair) {
    let mailgun_user = store.get("MAILGUN_USER").unwrap();
    let mailgun_token = store.get("MAILGUN_PASSWORD").unwrap();
    let mailgun_domain = store.get("MAILGUN_DOMAIN").unwrap();
    let admin_email = store.get("ADMIN_EMAIL").unwrap();
    std::env::set_var("ADMIN_EMAIL", admin_email);

    let keypair = JWTKeyPair {
        public_key: store.get("ACCESS_TOKEN_PUBLIC_KEY").unwrap(),
        private_key: store.get("ACCESS_TOKEN_PRIVATE_KEY").unwrap()
    };
    (mailgun_user, mailgun_token, mailgun_domain, keypair)
}

pub async fn run_migrations(db: Arc<Mutex<Connection>>) {
    let faery_repository = Arc::new(FaeryRepository::new(db.clone()));
    let player_repository = Arc::new(PlayerRepository::new(db.clone()));
    let manager = migrations::Manager::new(db, player_repository.clone(), faery_repository.clone());
    log::info!("Running migrations");
    manager.migrate().await.unwrap();
}
