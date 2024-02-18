use std::sync::Arc;
use libsql::Database;
use serde::{Deserialize, Serialize};
use crate::version::VERSION;
use semver::{Version, VersionReq};
use tokio::sync::Mutex;
use crate::faery::FaeryRepository;
use crate::repository::{Repository, RepositoryError, RepositoryItem, RepositoryResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct Migration {
    pub current_version: Option<Version>,
    pub target_version: Version,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct MigrationData {
    pub id: i64,
    pub current_version: Option<String>,
    pub target_version: Option<String>,
}

impl Clone for MigrationData {
    fn clone(&self) -> Self {
        MigrationData {
            id: self.id,
            current_version: self.current_version.clone(),
            target_version: self.target_version.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.id = source.id;
        self.current_version = source.current_version.clone();
        self.target_version = source.target_version.clone();
    }
}

impl From<MigrationData> for Migration {
    fn from(data: MigrationData) -> Self {
        Migration {
            current_version: data.current_version.map(|v| Version::parse(&v).unwrap()),
            target_version: data.target_version.map(|v| Version::parse(&v).unwrap()).unwrap_or_else(|| Version::parse(VERSION).unwrap()),
        }
    }
}

impl From<Migration> for MigrationData {
    fn from(migration: Migration) -> Self {
        MigrationData {
            id: 0,
            current_version: migration.current_version.map(|v| v.to_string()),
            target_version: Some(migration.target_version.to_string()),
        }
    }
}

impl Migration {
    // TODO: Remove naked `unwrap` calls
    pub fn new(current_version: Option<String>, target_version: Option<String>) -> Self {
        Migration {
            current_version: current_version
                .map(|v| Version::parse(&v).unwrap()),
            target_version: target_version
                .map(|v| Version::parse(&v).unwrap())
                .unwrap_or_else(|| Version::parse(VERSION).unwrap()),
        }
    }
}

impl RepositoryItem for MigrationData {
    fn masked_columns(_is_admin: bool) -> Vec<String> {
        vec!["id".to_string()]
    }

    fn saved_columns() -> Vec<String> {
        vec!["current_version".to_string(), "target_version".to_string()]
    }

    fn all_columns() -> Vec<String> {
        vec!["id".to_string(), "current_version".to_string(), "target_version".to_string()]
    }

    fn table_name() -> String {
        "migrations".to_string()
    }

}

#[derive(Clone)]
pub struct Manager {
    db: Arc<Mutex<Database>>,
    faery_repository: Arc<FaeryRepository>,
}

impl Manager {
    pub fn new(db: Arc<Mutex<Database>>, faery_repository: Arc<FaeryRepository>) -> Manager {
        Manager {
            db,
            faery_repository
        }
    }

    pub async fn needs_migration(&self) -> bool {
        log::info!("Checking for migrations");
        let migration_data = self.get(0).await.unwrap_or_else(|_| Migration::new(None, None).into());
        log::info!("Migration data: {:?}", migration_data);
        let current_migration: Migration = migration_data.into();
        match current_migration.current_version {
            Some(current_version) => {
                let version_req = VersionReq::parse(
                    &format!(">={}", current_migration.target_version.to_string())).unwrap();
                !version_req.matches(&current_version)
            },
            None => true,
        }
    }

    pub async fn migrate(&self) -> RepositoryResult<()> {
        // TODO: handle checking
        log::info!("Migrating 0 to 0.2.1");
        self.migrate_0_to_021().await
    }

    // Mark: - Migrations
    pub async fn migrate_0_to_021(&self) -> RepositoryResult<()> {
        log::info!("Creating migrations table");
        let result = self.create_table().await;
        match result {
            Ok(_) => {
                // Update the migration record to start the transaction of updates
                log::info!("Starting migration record");
                let result = self.save(
                    Migration::new(
                        // This is the first migration ever
                        Some("0.0.0".to_string()),
                        Some("0.2.1".to_string())
                    ).into()).await;
                match result {
                    Ok(_) => {
                        // TODO: Figure out a way to do this with transactions
                        // For now, just create the table as-is; 0.2.2 will add the update_table method
                        log::info!("Creating faery table");
                        match self.faery_repository.create_table().await {
                            Ok(_) => {
                                log::info!("Finalizing migration record");
                                match self.save(
                                    Migration::new(
                                        // Save the current version as the target version to indicate
                                        // that the migration has been completed
                                        Some("0.2.1".to_string()),
                                        Some("0.2.1".to_string())
                                    ).into()).await {
                                    Ok(_) => Ok(()),
                                    Err(_) => Err(RepositoryError::MigrationFailed(
                                        Version::parse("0.2.1").unwrap(),
                                        Version::parse("0.2.1").unwrap(),
                                    )),
                                }
                            },
                            Err(_) => Err(RepositoryError::MigrationFailed(
                                Version::parse("0.2.1").unwrap(),
                                Version::parse("0.2.1").unwrap(),
                            ))
                        }
                    },
                    Err(_) => Err(RepositoryError::MigrationFailed(
                        Version::parse("0.0.0").unwrap(),
                        Version::parse("0.2.1").unwrap(),
                    )),
                }
            },
            Err(_) => Err(RepositoryError::MigrationFailed(
                Version::parse("0.0.0").unwrap(),
                Version::parse("0.2.1").unwrap(),
            )),
        }
    }
}

#[shuttle_runtime::async_trait]
impl Repository for Manager {
    type Item = MigrationData;
    type RowIdentifier = i64;

    async fn create(&self, template_item: Option<MigrationData>) -> RepositoryResult<i64> {
        match template_item {
            Some(template_item) => self.save(template_item).await,
            None => Ok(0),
        }
    }

    async fn save(&self, item: MigrationData) -> RepositoryResult<i64> {
        let db = self.db.lock().await.connect().unwrap();
        log::info!("Saving migration: {:?}", item);
        let insert_item = item.clone();
        let result = db.execute(
            "INSERT INTO migrations (id, current_version, target_version) VALUES (0, ?1, ?2)",
            (insert_item.current_version, insert_item.target_version)
        ).await;
        match result {
            Ok(_) => Ok(0),
            Err(_) => {
                // Try update
                let update_item = item.clone();
                let result = db.execute(
                    "UPDATE migrations SET current_version = ?1, target_version = ?2 WHERE id = 0",
                    (update_item.current_version, update_item.target_version)
                ).await;
                match result {
                    Ok(_) => Ok(0),
                    Err(_) => Err(RepositoryError::Other),
                }
            },
        }
    }

    async fn get(&self, _: i64) -> RepositoryResult<MigrationData> {
        let db = self.db.lock().await.connect().unwrap();
        let result = db.query(
            "SELECT current_version, target_version FROM migrations WHERE id = 0",
            ()
        ).await;
        match result {
            Ok(mut rows) => {
                let result = rows.next().await;
                match result {
                    Ok(Some(row)) => {
                        let current_version: String = row.get(0).unwrap();
                        let target_version: String = row.get(1).unwrap();
                        Ok(MigrationData {
                            id: 0,
                            current_version: Some(current_version),
                            target_version: Some(target_version),
                        })
                    },
                    Ok(None) => Err(RepositoryError::NotFound),
                    Err(_) => Err(RepositoryError::Other),
                    // None => Err(RepositoryError::NotFound),
                }
            },
            Err(_) => Err(RepositoryError::Other),
        }
    }

    async fn get_all(&self) -> RepositoryResult<Vec<MigrationData>> {
        Ok(vec![])
    }

    async fn delete(&self, _id: i64) -> RepositoryResult<()> {
        Ok(())
    }

    async fn create_table(&self) -> RepositoryResult<()> {
        let db = self.db.lock().await.connect().unwrap();
        let result = db.execute(
            "CREATE TABLE IF NOT EXISTS migrations (
                    id INTEGER PRIMARY KEY CHECK (id = 0),
                    current_version VARCHAR(255) NOT NULL,
                    target_version VARCHAR(255) NOT NULL
                )",
            ()
        ).await;
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(RepositoryError::Other),
        }
    }

    async fn drop_table(&self) -> RepositoryResult<()> {
        Ok(())
    }
}


impl From<Version> for Migration {
    fn from(version: Version) -> Self {
        Migration::new(None, Some(version.to_string()))
    }
}