mod faery;
mod sql;
mod dross;
mod endpoints;

use std::net::SocketAddr;
use axum::{routing::get, Router};
use tower_http::services::ServeDir;
use libsql::{Builder, Connection};
use std::sync::Arc;
use tokio::sync::Mutex;
use sql::SqlModel;

pub struct DrossManagerService {
    db: Arc<Mutex<Connection>>,
    router: Router
}
pub struct DrossManagerState {
    pub db: Arc<Mutex<Connection>>,
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
    //
    // turso.execute_batch("DROP TABLE IF EXISTS faeries").await.unwrap();

    turso.execute_batch(
        faery::Faery::generate_sql_create_table().as_str()
    ).await.unwrap();

    let db = Arc::new(Mutex::new(turso));
    let state = Arc::new(DrossManagerState {
        db: db.clone()
    });

    let router = Router::new()
        .route("/hello", get(hello_world))
        .route("/faeries", get(endpoints::list_faeries))
        .with_state(state)
        .nest_service("/", ServeDir::new("public_html"));

    Ok(DrossManagerService {
        db,
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
        Faery::new("Tinkerbell".to_string(), "me@example.com".to_string())
    }

    fn new_faery_two() -> Faery {
        Faery::new("Silvermist".to_string(), "you@example.com".to_string())
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