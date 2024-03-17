use std::sync::Arc;
use chrono::{Duration, Utc};
use libsql::{Connection, params};
use rand::distributions::Alphanumeric;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::repository::{Repository, RepositoryError, RepositoryItem, RepositoryResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub(crate) id: Option<i64>,
    pub first_name: String,
    pub last_name: String,
    pub auth_email: String,
    pub auth_token: Option<String>,
    pub auth_token_expires: Option<i64>,
    pub mailing_address: String,
    pub is_admin: bool,
}

struct TokenData {
    token: String,
    expires: i64
}

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;


// let token = TokenData::new(user_id);

impl TokenData {
    pub fn new(user_id: i64) -> TokenData {
        let expiration_date = Utc::now() + Duration::try_minutes(15).unwrap();
        // use user_id as a seed for rand to generate a large 64-bit number
        let rng = ChaCha8Rng::seed_from_u64(user_id as u64);
        let r = rng.sample_iter(&Alphanumeric).take(64).collect::<Vec<_>>();
        let token = String::from_utf8_lossy(&r).to_string();
        TokenData {
            token: format!("{}-token", token),
            expires: expiration_date.timestamp_millis()
        }
    }
}

impl Model {
    pub fn new(
        id: Option<i64>,
        first_name: String,
        last_name: String,
        auth_email: String,
        auth_token: Option<String>,
        auth_token_expires: Option<i64>,
        mailing_address: String,
        is_admin: bool,
    ) -> Model {
        Model {
            id,
            first_name,
            last_name,
            auth_email,
            auth_token,
            auth_token_expires,
            mailing_address,
            is_admin,
        }
    }

    pub fn from_response(row: &libsql::Row) -> Model {
        Model {
            id: row.get(0).unwrap(),
            first_name: row.get(1).unwrap(),
            last_name: row.get(2).unwrap(),
            auth_email: row.get(3).unwrap(),
            auth_token: row.get(4).unwrap(),
            auth_token_expires: row.get(5).unwrap(),
            mailing_address: row.get(6).unwrap(),
            is_admin: row.get(7).unwrap(),
        }
    }
}

impl RepositoryItem for Model {
    fn masked_columns(_is_admin: bool) -> Vec<String> {
        // TODO: Implement masking with responses
        vec![]
    }

    fn saved_columns() -> Vec<String> {
        vec![
            "id".to_string(),
            "first_name".to_string(),
            "last_name".to_string(),
            "auth_email".to_string(),
            "mailing_address".to_string(),
            "is_admin".to_string()
        ]
    }

    fn all_columns() -> Vec<String> {
        vec![
            "id".to_string(),
            "first_name".to_string(),
            "last_name".to_string(),
            "auth_email".to_string(),
            "auth_token".to_string(),
            "auth_token_expires".to_string(),
            "mailing_address".to_string(),
            "is_admin".to_string()
        ]
    }

    fn table_name() -> String where Self: Sized {
        "players".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pub id: Option<i64>,
    pub first_name: String,
    pub last_name: String,
    pub auth_email: String,
    pub mailing_address: String,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    // Can't be optional for login
    pub id: i64,
    pub auth_token: String,
    pub auth_token_expires: i64,
    pub is_admin: bool
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerClaim {
    pub id: i64,
    pub is_admin: bool,
}

impl From<LoginResponse> for PlayerClaim {
    fn from(response: LoginResponse) -> Self {
        PlayerClaim {
            id: response.id,
            is_admin: response.is_admin,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerResponse {
    pub player: PlayerData,
}

impl From<Model> for PlayerData {
    fn from(model: Model) -> Self {
        PlayerData {
            id: model.id,
            first_name: model.first_name,
            last_name: model.last_name,
            auth_email: model.auth_email,
            mailing_address: model.mailing_address,
            is_admin: model.is_admin,
        }
    }
}

impl From<Model> for PlayerResponse {
    fn from(model: Model) -> Self {
        PlayerResponse {
            player: PlayerData::from(model),
        }
    }
}

impl From<Model> for LoginResponse {
    fn from(model: Model) -> Self {
        LoginResponse {
            id: model.id.unwrap(),
            auth_token: model.auth_token.unwrap(),
            auth_token_expires: model.auth_token_expires.unwrap(),
            is_admin: model.is_admin,
        }
    }
}

impl From<Model> for PlayerRequest {
    fn from(model: Model) -> Self {
        PlayerRequest {
            first_name: model.first_name,
            last_name: model.last_name,
            auth_email: model.auth_email,
            mailing_address: model.mailing_address,
        }
    }
}

impl From<PlayerRequest> for Model {
    fn from(request: PlayerRequest) -> Self {
        Model {
            id: None,
            first_name: request.first_name,
            last_name: request.last_name,
            auth_email: request.auth_email,
            auth_token: None,
            auth_token_expires: None,
            mailing_address: request.mailing_address,
            is_admin: false,
        }
    }
}

impl From<PlayerUpdateRequest> for Model {
    fn from(request: PlayerUpdateRequest) -> Self {
        Model {
            id: None,
            first_name: request.first_name,
            last_name: request.last_name,
            auth_email: request.auth_email,
            auth_token: None,
            auth_token_expires: None,
            mailing_address: request.mailing_address,
            is_admin: request.is_admin,
        }
    }

}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRequest {
    pub first_name: String,
    pub last_name: String,
    pub auth_email: String,
    pub mailing_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerUpdateRequest {
    pub first_name: String,
    pub last_name: String,
    pub auth_email: String,
    pub mailing_address: String,
    pub is_admin: bool,
}

pub struct PlayerRepository {
    db: Arc<Mutex<Connection>>,
}

impl PlayerRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> PlayerRepository {
        PlayerRepository {
            db,
        }
    }

    pub async fn generate_token(&self, id: Option<i64>) -> RepositoryResult<String> {
        let db = self.db.lock().await;
        let id = match id {
            Some(id) => id,
            None => return Err(RepositoryError::NotFound),
        };
        let token_data = TokenData::new(id);
        match db.execute("UPDATE players SET auth_token = ?1, auth_token_expires = ?2 WHERE id = ?3", params![
            token_data.token.clone(),
            token_data.expires,
            id
        ]).await {
            Ok(_) => Ok(token_data.token),
            Err(_) => Err(RepositoryError::Other),
        }
    }

    async fn validate_token(&self, token: String) -> RepositoryResult<i64> {
        let db = self.db.lock().await;
        // TODO: validate expiration timestamp
        match db.query("SELECT id FROM players WHERE auth_token = ?1", params![token]).await?.next()? {
            Some(row) => {
                let id: i64 = row.get(0).unwrap();
                Ok(id)
            },
            None => Err(RepositoryError::NotFound),
        }
    }

    pub async fn login(&self, email: String, token: String) -> RepositoryResult<LoginResponse> {
        log::info!("Logging in with email: {}", email);
        // scoped to prevent deadlocks later
        let uid: Option<i64> = {
            let db = self.db.lock().await;
            let mut user_id_stmt = db.prepare("SELECT id FROM players WHERE auth_email = ?1").await.unwrap();
            log::info!("Validating user ID");

            match user_id_stmt.query(params![email.clone()]).await?.next()? {
                Some(row) => row.get(0).unwrap_or(None),
                None => None,
            }
        };

        log::info!("Validating token...");
        let token_id = self.validate_token(token.clone()).await?;
        if uid.unwrap() != token_id {
            // short-circuit the login process if the token doesn't match the user
            return Err(RepositoryError::NotFound);
        }
        let db = self.db.lock().await;
        // this validates the id, email and token existing in the same row (valid login)
        let mut stmt = db.prepare("SELECT * FROM players WHERE id = ?1 AND auth_email = ?2 AND auth_token = ?3").await.unwrap();
        match stmt.query(params![token_id, email, token]).await?.next()? {
            Some(row) => {
                let player = Model::from_response(&row);
                // Ensure the token hasn't expired
                return if validate_token_age(player.auth_token_expires.unwrap()) {
                    Ok(player.into())
                } else {
                    Err(RepositoryError::ExpiredGuard)
                }
            },
            None => Err(RepositoryError::NotFound),
        }
    }

    pub async fn get_by_email(&self, email: String) -> RepositoryResult<Model> {
        let db = self.db.lock().await;
        let mut stmt = db.prepare("SELECT * FROM players WHERE auth_email = ?1").await.unwrap();
        let mut res = stmt.query(params![email]).await.unwrap();
        match res.next().unwrap() {
            Some(row) => {
                let player = Model::from_response(&row);
                Ok(player)
            },
            None => Err(RepositoryError::NotFound),
        }
    }

    pub async fn admin_count(&self) -> RepositoryResult<i64> {
        let db = self.db.lock().await;
        match db.query("SELECT COUNT(is_admin) from players", ()).await?.next()? {
            Some(row) => {
                let count: i64 = row.get(0).unwrap();
                Ok(count)
            },
            None => Err(RepositoryError::Other),
        }
    }
}
fn validate_token_age(expiration: i64) -> bool {
    // validates the expiration date is not in the past
    let expiration_date: chrono::DateTime<Utc> = chrono::DateTime::<Utc>::from_timestamp_millis(expiration).unwrap();
    let now = Utc::now();
    now < expiration_date
}
#[shuttle_runtime::async_trait]
impl Repository for PlayerRepository {
    type Item = Model;
    type RowIdentifier = i64;

    async fn save(&self, player: Model) -> RepositoryResult<i64> {
        let db = self.db.lock().await;
        let result = match player.id {
            Some(id) => {
                let mut stmt = db.prepare("UPDATE players SET first_name = ?1, last_name = ?2, auth_email = ?3, auth_token = ?4, auth_token_expires = ?5, mailing_address = ?6, is_admin = ?7 WHERE id = ?8").await.unwrap();
                stmt.query(params![
                    player.first_name,
                    player.last_name,
                    player.auth_email,
                    player.auth_token,
                    player.auth_token_expires,
                    player.mailing_address,
                    player.is_admin,
                    id
                ]).await
            },
            None => {
                // We'll let a custom method handle auth token data
                let mut stmt = db.prepare("INSERT INTO players (first_name, last_name, auth_email, mailing_address, is_admin) VALUES (?1, ?2, ?3, ?4, ?5)").await.unwrap();
                stmt.query(params![
                    player.first_name,
                    player.last_name,
                    player.auth_email,
                    player.mailing_address,
                    player.is_admin
                ]).await
            },
        };
        match result {
            Ok(_) => Ok(db.last_insert_rowid()),
            Err(_) => Err(RepositoryError::Other),
        }
    }

    async fn get(&self, id: i64) -> RepositoryResult<Model> {
        let db = self.db.lock().await;
        let mut stmt = db
            .prepare("SELECT * FROM players WHERE id = ?1").await
            .unwrap();
        let mut res = stmt.query([id]).await.unwrap();
        match res.next().unwrap() {
            Some(row) => {
                let player = Model::from_response(&row);
                Ok(player)
            },
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn get_all(&self) -> RepositoryResult<Vec<Model>> {
        todo!()
    }

    async fn delete(&self, _id: i64) -> RepositoryResult<()> {
        todo!()
    }

    async fn create_table(&self) -> RepositoryResult<()> {
        let db = self.db.lock().await;
        let result = db.execute(
            r#"CREATE TABLE IF NOT EXISTS players (
    id INTEGER PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    auth_email TEXT NOT NULL,
    auth_token TEXT,
    auth_token_expires INTEGER,
    mailing_address TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL
)"#, ()).await;
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(RepositoryError::Other),
        }
    }

    async fn drop_table(&self) -> RepositoryResult<()> {
        todo!()
    }
}