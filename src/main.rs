mod dross;
mod version;
mod prelude;
mod repository;
mod endpoints;
mod middleware;
mod service;

use std::sync::Arc;
use libsql::Connection;
use shuttle_axum::ShuttleAxum;
use tokio::sync::Mutex;
use prelude::*;

use tower_sessions::session_store::ExpiredDeletion;
use crate::service::DrossManagerState;

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_secrets::Secrets] store: shuttle_secrets::SecretStore,
    #[shuttle_turso::Turso(
        addr = "{secrets.TURSO_URL}",
        token = "{secrets.TURSO_TOKEN}"
    )] turso: Connection
) -> ShuttleAxum {

    let (mailgun_user, mailgun_token, mailgun_domain, keypair) = service::setup_environment(&store);

    let db = Arc::new(Mutex::new(turso.clone()));
    let faery_repository = Arc::new(FaeryRepository::new(db.clone()));
    let player_repository = Arc::new(PlayerRepository::new(db.clone()));
    let email_repository = Arc::new(EmailRepository::new(mailgun_user, mailgun_token, mailgun_domain));

    let state = Arc::new(DrossManagerState {
        jwt_key_pair: keypair
    });

    // TODO: Migrate to a migration service and / or remove the need to pass in the repositories like this
    service::run_migrations(db.clone()).await;

    let router = service::create_router_tree(
        faery_repository,
        email_repository,
        player_repository,
        state,
        turso).await;
    Ok(router.into())
}