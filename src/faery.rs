use std::sync::Arc;
use libsql::{Connection, Row};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::dross::{DrossError, DrossHolder, DrossResult};
use libsql::params;

// TODO: move
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub enum RepositoryError {
    NotFound,
    AlreadyExists,
    Other,
}
pub type RepositoryResult = Result<(), RepositoryError>;
pub type FaeryResult = Result<Faery, RepositoryError>;
// TODO: fix this
pub type FaeriesResult = Result<Vec<Faery>, RepositoryError>;

// TODO: fix this
#[shuttle_runtime::async_trait]
pub trait Repository: Sized {
    async fn create(&self, faery: Option<Faery>) -> RepositoryResult;
    async fn save(&self, faery: Faery) -> RepositoryResult;
    async fn create_table(&self) -> RepositoryResult;
    async fn drop_table(&self) -> RepositoryResult;
    async fn get(&self, id: u32) -> FaeryResult;
    async fn get_all(&self) -> FaeriesResult;
    fn table_name() -> String where Self: Sized;
}
#[derive(Clone)]
pub struct FaeryRepository {
    db: Arc<Mutex<Connection>>,
}

impl FaeryRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> FaeryRepository {
        FaeryRepository {
            db,
        }
    }
}

#[shuttle_runtime::async_trait]
impl Repository for FaeryRepository {
    // Mark: Repository
    async fn create(&self, faery: Option<Faery>) -> RepositoryResult {
        match faery {
            Some(faery) => {
                self.save(faery).await
            },
            None => Err(RepositoryError::Other),
        }
    }

    async fn save(&self, faery: Faery) -> RepositoryResult {
        // TODO: implement upsert
        let db = self.db.lock().await;
        let mut stmt = db.prepare("INSERT INTO faeries (name, is_admin, email, dross) VALUES (?1, ?2, ?3, ?4)").await.unwrap();
        let result = stmt.query(params![faery.name, faery.is_admin, faery.email, faery.dross]).await;
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(RepositoryError::Other),
        }
    }

    async fn create_table(&self) -> RepositoryResult {
        let db = self.db.lock().await;
        let result = db.execute(
            r#"CREATE TABLE IF NOT EXISTS faeries (
    id INTEGER PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    is_admin BOOLEAN NOT NULL,
    email VARCHAR(255) NOT NULL,
    dross INTEGER
)"#, ()).await;
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(RepositoryError::Other),
        }
    }

    async fn drop_table(&self) -> RepositoryResult {
        let db = self.db.lock().await;
        let result = db.execute("DROP TABLE IF EXISTS faeries", ()).await;
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(RepositoryError::Other),
        }
    }

    // Mark: Faery
    async fn get(&self, id: u32) -> FaeryResult {
        let db = self.db.lock().await;
        let mut stmt = db
            .prepare("SELECT * FROM faeries WHERE id = ?1")
            .await
            .unwrap();
        let mut res = stmt.query([id]).await.unwrap();
        match res.next().await.unwrap() {
            Some(row) => {
                let faery = Faery::from_response(&row);
                Ok(faery)
            },
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn get_all(&self) -> FaeriesResult {
        let db = self.db.lock().await;
        let mut res = db.query("SELECT * FROM faeries", ()).await.unwrap();
        let mut faeries: Vec<Faery> = Vec::new();
        while let Some(row) = res.next().await.unwrap() {
            faeries.push(Faery::from_response(&row));
        }
        Ok(faeries)
    }

    fn table_name() -> String {
        "faeries".to_string()
    }
}

// Faery represents the user of the application.
// It has the name of the user, their email, an authentication token, and a count of their dross.
#[derive(Debug, Deserialize, Serialize)]
pub struct Faery {
    pub(crate) id: Option<u32>,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub is_admin: bool,
    #[serde(skip_serializing)]
    pub auth_token: Option<String>,
    pub dross: u32,
}

#[allow(dead_code)]
impl Faery {
    // This is a method that creates a new Faery.
    // It takes a name and an email and returns a Faery.
    pub fn new(name: String, email: String, is_admin: bool, dross: u32, id: Option<u32>) -> Faery {
        Faery {
            id,
            name,
            email,
            is_admin,
            auth_token: None,
            dross,
        }
    }

    pub fn from_response(row: &Row) -> Faery {
        Faery::new(
            row.get(1).unwrap(),
            row.get(3).unwrap(),
            row.get(2).unwrap(),
            row.get(4).unwrap(),
            row.get(0).unwrap_or(None),
        )
    }

    // This is a method that returns the name of the Faery.
    pub fn name(&self) -> &str {
        &self.name
    }

    // This is a method that returns the email of the Faery.
    pub fn email(&self) -> &str {
        &self.email
    }

    // This is a method that returns whether the Faery is an admin.
    pub fn is_admin(&self) -> bool {
        self.is_admin
    }

    // This is a method that returns the dross of the Faery.
    pub fn dross(&self) -> u32 {
        self.dross
    }

    // This is a method that returns the auth token of the Faery.
    pub fn auth_token(&self) -> Option<&str> {
        self.auth_token.as_deref()
    }

    // This is a method that sets the auth token of the Faery.
    pub fn set_auth_token(&mut self, auth_token: String) {
        self.auth_token = Some(auth_token);
    }
}

impl DrossHolder for Faery {
    // This is a method that increments the dross of the Faery.
    fn increment_dross(&mut self, amount: u32) -> DrossResult {
        if amount <= 0 {
            return Err(DrossError::InvalidIncrement);
        }
        self.dross += amount;
        Ok(self.dross)
    }

    // This is a method that decrements the dross of the Faery.
    fn decrement_dross(&mut self, amount: u32) -> DrossResult {
        match amount {
            0 => Err(DrossError::InvalidDecrement),
            _ if amount > self.dross => Err(DrossError::NotEnoughDross),
            _ => {
                self.dross -= amount;
                Ok(self.dross)
            }
        }
    }

    // This is a method that returns the dross of the Faery.
    fn dross(&self) -> DrossResult {
        Ok(self.dross)
    }
}
