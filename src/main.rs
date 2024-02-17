mod faery;
mod sql;
mod dross;
mod endpoints;

use std::net::SocketAddr;
use axum::{routing::get, Router};
use tower_http::services::ServeDir;
use libsql::Builder;
use std::sync::Arc;
use tokio::sync::Mutex;
use faery::Repository;
use http::{Request, Response, Method, header};
use bytes::Bytes;
use tower::{ServiceBuilder, ServiceExt, Service};
use tower_http::cors::{Any, CorsLayer};
use std::convert::Infallible;

pub struct DrossManagerService {
    router: Router
}
pub struct DrossManagerState {
    pub faery_repository: Arc<faery::FaeryRepository>
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_secrets::Secrets] store: shuttle_secrets::SecretStore,
    // #[shuttle_turso::Turso(
    //     addr = "",
    //     token = ""
    // )] turso: Connection
) -> Result<DrossManagerService, shuttle_runtime::Error> {

    let turso_addr = store.get("TURSO_URL").unwrap();
    let turso_token = store.get("TURSO_TOKEN").unwrap();
    let db = Builder::new_remote(turso_addr, turso_token).build().await.unwrap();

    let turso = db.connect().unwrap();

    let db = Arc::new(Mutex::new(turso));
    let state = Arc::new(DrossManagerState {
        faery_repository: Arc::new(faery::FaeryRepository::new(db.clone())),
    });

    // TODO: Remove everything except create_table
    state.faery_repository.drop_table().await.unwrap();
    state.faery_repository.create_table().await.unwrap();
    state.faery_repository.save(faery::Faery::new(
            "NightWater".to_string(), "example@arikel.net".to_string(), true, 0, None)
        )
        .await.unwrap();

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let router = Router::new()
        .route("/hello", get(hello_world))
        .route("/faeries", get(endpoints::list_faeries))
        .route("/faeries/:faery_id", get(endpoints::get_faery))
        .layer(ServiceBuilder::new().layer(cors))
        .with_state(state)
        .nest_service("/", ServeDir::new("dross-manager-frontend/dist"));

    Ok(DrossManagerService {
        router
    })
}

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
    use crate::faery::Faery;

    fn new_faery() -> Faery {
        Faery::new("Tinkerbell".to_string(), "me@example.com".to_string(), false, 0, None)
    }

    fn new_faery_two() -> Faery {
        Faery::new("Silvermist".to_string(), "you@example.com".to_string(), false, 0, None)
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