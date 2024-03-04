use std::sync::Arc;
use libsql::{Row, params, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::dross::{DrossError, DrossHolder, DrossResult};
use crate::repository::{Repository, RepositoryError, RepositoryItem, RepositoryResult};

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

impl RepositoryItem for Model {
    fn masked_columns(is_admin: bool) -> Vec<String> {
        let mut columns = vec![];
        if !is_admin {
            columns.push("email".to_string());
        }
        columns
    }

    fn saved_columns() -> Vec<String> {
        // get all columns
        let columns = Model::all_columns();
        // filter out masked columns, assuming is_admin is true
        let masked_columns = Model::masked_columns(true);
        columns.into_iter().filter(|c| !masked_columns.contains(c)).collect()
    }

    fn all_columns() -> Vec<String> {
        vec![
            "id".to_string(),
            "name".to_string(),
            "is_admin".to_string(),
            "email".to_string(),
            "dross".to_string(),
        ]
    }

    fn table_name() -> String {
        "faeries".to_string()
    }

}

#[shuttle_runtime::async_trait]
// Mark: Repository
impl Repository for FaeryRepository {
    type Item = Model;
    type RowIdentifier = i64;


    async fn save(&self, faery: Model) -> RepositoryResult<i64> {
        let db = self.db.lock().await;
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
            Ok(_) => Ok(db.last_insert_rowid()),
            Err(_) => Err(RepositoryError::Other),
        }
    }

    // Mark: Faery
    async fn get(&self, id: i64) -> RepositoryResult<Model> {
        let db = self.db.lock().await;
        let mut stmt = db
            .prepare("SELECT * FROM faeries WHERE id = ?1")
            .await
            .unwrap();
        let mut res = stmt.query([id]).await.unwrap();
        match res.next().unwrap() {
            Some(row) => {
                let faery = Model::from_response(&row);
                Ok(faery)
            },
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn get_all(&self) -> RepositoryResult<Vec<Model>> {
        let db = self.db.lock().await;
        let result = db.query("SELECT * FROM faeries", ()).await;
        let mut res = match result {
            Ok(res) => res,
            Err(err) => {
                log::error!("Error getting all faeries: {:?}", err);
                return Err(RepositoryError::Other)
            },
        };

        let mut faeries: Vec<Model> = Vec::new();
        while let Ok(result_row) = res.next() {
            match result_row {
                Some(row) => {
                    faeries.push(Model::from_response(&row));
                },
                None => break,
            }
        }
        Ok(faeries)
    }

    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        let db = self.db.lock().await;
        let result = db.execute("DELETE FROM faeries WHERE id = ?1", [id]).await;
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(RepositoryError::NotFound),
        }
    }

    async fn create_table(&self) -> RepositoryResult<()> {
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

    async fn drop_table(&self) -> RepositoryResult<()> {
        let db = self.db.lock().await;
        let result = db.execute("DROP TABLE IF EXISTS faeries", ()).await;
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(RepositoryError::Other),
        }
    }
}

// Faery represents the user of the application.
// It has the name of the user, their email, an authentication token, and a count of their dross.
#[derive(Debug, Deserialize, Serialize)]
pub struct Model {
    pub(crate) id: Option<i64>,
    pub name: String,
    pub email: String,
    // TODO: deprecated
    pub is_admin: bool,
    pub dross: u32,
}

#[allow(dead_code)]
impl Model {
    // This is a method that creates a new Faery.
    // It takes a name and an email and returns a Faery.
    pub fn new(name: String, email: String, is_admin: bool, dross: u32, id: Option<i64>) -> Model {
        Model {
            id,
            name,
            email,
            is_admin,
            dross,
        }
    }

    pub fn from_response(row: &Row) -> Model {
        Model::new(
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
}

impl Clone for Model {
    fn clone(&self) -> Self {
        Model {
            id: self.id,
            name: self.name.clone(),
            email: self.email.clone(),
            is_admin: self.is_admin,
            dross: self.dross,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.id = source.id;
        self.name = source.name.clone();
        self.email = source.email.clone();
        self.is_admin = source.is_admin;
        self.dross = source.dross;
    }
}

impl DrossHolder for Model {
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


#[derive(Deserialize, Debug)]
pub struct CreateFaeryRequest {
    pub name: String,
    pub email: String,
}

impl From<CreateFaeryRequest> for Model {
    fn from(req: CreateFaeryRequest) -> Self {
        Model::new(req.name, req.email, false, 0, None)
    }
}