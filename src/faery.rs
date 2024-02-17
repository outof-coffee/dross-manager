use std::sync::Arc;
use libsql::{Row, params, Database};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::dross::{DrossError, DrossHolder, DrossResult};
use crate::repository::{Repository, RepositoryError, RepositoryResult};

#[derive(Clone)]
pub struct FaeryRepository {
    db: Arc<Mutex<Database>>,
}

impl FaeryRepository {
    pub fn new(db: Arc<Mutex<Database>>) -> FaeryRepository {
        FaeryRepository {
            db,
        }
    }
}

#[shuttle_runtime::async_trait]
// Mark: Repository
impl Repository for FaeryRepository {
    type Item = Faery;

    async fn create(&self, faery: Option<Faery>) -> RepositoryResult<()> {
        match faery {
            Some(faery) => {
                self.save(faery).await
            },
            None => Err(RepositoryError::Other),
        }
    }

    async fn save(&self, faery: Faery) -> RepositoryResult<()> {
        let db = self.db.lock().await.connect().unwrap();
        let result = match faery.id {
            Some(id) => {
                let mut stmt = db.prepare("UPDATE faeries SET name = ?1, is_admin = ?2, email = ?3, dross = ?4 WHERE id = ?5").await.unwrap();
                stmt.query(params![faery.name, faery.is_admin, faery.email, faery.dross, id]).await
            },
            None => {
                let mut stmt = db.prepare("INSERT INTO faeries (name, is_admin, email, dross) VALUES (?1, ?2, ?3, ?4)").await.unwrap();
                stmt.query(params![faery.name, faery.is_admin, faery.email, faery.dross]).await
            },
        };
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(RepositoryError::Other),
        }
    }

    async fn create_table(&self) -> RepositoryResult<()> {
        let db = self.db.lock().await.connect().unwrap();
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

    async fn drop_table(&self) -> RepositoryResult<()> {
        let db = self.db.lock().await.connect().unwrap();
        let result = db.execute("DROP TABLE IF EXISTS faeries", ()).await;
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(RepositoryError::Other),
        }
    }

    // Mark: Faery
    async fn get(&self, id: u32) -> RepositoryResult<Faery> {
        let db = self.db.lock().await.connect().unwrap();
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

    async fn get_all(&self) -> RepositoryResult<Vec<Faery>> {
        let db = self.db.lock().await.connect().unwrap();
        let result = db.query("SELECT * FROM faeries", ()).await;
        let mut res = match result {
            Ok(res) => res,
            Err(err) => {
                log::error!("Error getting all faeries: {:?}", err);
                return Err(RepositoryError::Other)
            },
        };
        let mut faeries: Vec<Faery> = Vec::new();
        while let Ok(result_row) = res.next().await {
            match result_row {
                Some(row) => {
                    faeries.push(Faery::from_response(&row));
                },
                None => break,
            }
        }
        Ok(faeries)
    }

    async fn delete(&self, id: u32) -> RepositoryResult<()> {
        let db = self.db.lock().await.connect().unwrap();
        let result = db.execute("DELETE FROM faeries WHERE id = ?1", [id]).await;
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(RepositoryError::NotFound),
        }
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

impl Clone for Faery {
    fn clone(&self) -> Self {
        Faery {
            id: self.id,
            name: self.name.clone(),
            email: self.email.clone(),
            is_admin: self.is_admin,
            auth_token: self.auth_token.clone(),
            dross: self.dross,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.id = source.id;
        self.name = source.name.clone();
        self.email = source.email.clone();
        self.is_admin = source.is_admin;
        self.auth_token = source.auth_token.clone();
        self.dross = source.dross;
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
