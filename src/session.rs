use std::sync::Arc;
use libsql::{Database, params};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::repository::{Repository, RepositoryError, RepositoryItem, RepositoryResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct Session {
    pub user_id: i64,
    pub session_token: String,
    pub expires_in: i64,
}

pub struct SessionRepository {
    pub db: Arc<Mutex<Database>>,
}

impl RepositoryItem for Session {
    fn masked_columns(_: bool) -> Vec<String> {
        vec![]
    }

    fn saved_columns() -> Vec<String> {
        vec!["user_id".to_string(), "session_token".to_string(), "expires_in".to_string()]
    }

    fn all_columns() -> Vec<String> {
        vec!["id".to_string(), "user_id".to_string(), "session_token".to_string(), "expires_in".to_string()]
    }

    fn table_name() -> String where Self: Sized {
        "sessions".to_string()
    }
}

#[shuttle_runtime::async_trait]
impl Repository for SessionRepository {
    type Item = Session;
    type RowIdentifier = i64;

    async fn create(&self, template_item: Option<Session>) -> RepositoryResult<i64> {
        todo!()
    }

    async fn save(&self, item: Session) -> RepositoryResult<i64> {
        todo!()
    }

    async fn get(&self, id: i64) -> RepositoryResult<Session> {
        todo!()
    }

    async fn get_all(&self) -> RepositoryResult<Vec<Session>> {
        // Will never be used
        todo!()
    }

    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        todo!()
    }

    async fn create_table(&self) -> RepositoryResult<()> {
        let db = self.db.lock().await.connect().unwrap();
        let mut stmts = vec![];
        stmts.push("BEGIN".to_string());
        stmts.push("CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            session_token TEXT NOT NULL,
            expires_in INTEGER NOT NULL
        )".to_string());
        stmts.push("CREATE UNIQUE INDEX IF NOT EXISTS user_id_token_idx ON sessions (user_id, session_token)".to_string());
        stmts.push("COMMIT".to_string());

        let stmts = stmts.join(";");
        match db.execute_batch(&stmts).await {
            Ok(_) => Ok(()),
            Err(_) => Err(RepositoryError::Other)
        }
    }

    async fn drop_table(&self) -> RepositoryResult<()> {
        todo!()
    }
}

impl SessionRepository {
    pub async fn clean_up_expired(&self) -> RepositoryResult<()> {
        let db = self.db.lock().await.connect().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        let mut stmt = db.prepare("DELETE FROM sessions WHERE expires_in < ?").await.unwrap();
        match stmt.query(params![now]).await.unwrap() {
            _ => Ok(()),
        }
    }
}