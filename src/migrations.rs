use std::sync::Arc;
use libsql::{Database, params};
use serde::{Deserialize, Serialize};
use crate::version::VERSION;
use semver::{Version, VersionReq};
use tokio::sync::Mutex;
use crate::faery::FaeryRepository;
use crate::player;
use crate::player::PlayerRepository;
use crate::repository::{Repository, RepositoryError, RepositoryItem, RepositoryResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct Migration {
    pub current_version: Option<Version>,
    pub target_version: Version,
}

impl From<Migration> for RepositoryError {
    fn from(err: Migration) -> Self {
        RepositoryError::MigrationFailed(
            err.current_version.unwrap_or(Version::parse("0.0.0").unwrap()),
            err.target_version,
        )
    }
}

impl Clone for Migration {
    fn clone(&self) -> Self {
        Migration {
            current_version: self.current_version.clone(),
            target_version: self.target_version.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.current_version = source.current_version.clone();
        self.target_version = source.target_version.clone();
    }

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
    player_repository: Arc<PlayerRepository>,
    faery_repository: Arc<FaeryRepository>,
}

impl Manager {
    pub fn new(db: Arc<Mutex<Database>>, player_repository: Arc<PlayerRepository>, faery_repository: Arc<FaeryRepository>) -> Manager {
        Manager {
            db,
            player_repository,
            faery_repository
        }
    }

    pub async fn needs_migration(&self) -> bool {
        log::info!("Checking for migrations");
        let migration_data = self.get(0).await.unwrap_or_else(|_| Migration::new(None, None).into());
        log::info!("Migration data: {:?}", migration_data);
        match migration_data.current_version {
            Some(current_version) => {
                let version_req = VersionReq::parse(
                    &format!(">={}", VERSION.to_string())).unwrap();
                !version_req.matches(&Version::parse(&current_version).unwrap())
            },
            None => {
                return true;
            }
        }
    }

    pub async fn migrate(&self) -> RepositoryResult<()> {
        let current_state: Migration = self.get(0).await.unwrap_or_else(|_| Migration::new(None, None).into()).into();
        log::info!("Migrating to {}", VERSION.to_string());
        let current_migration: Migration = Migration::new(
            current_state.current_version.map(|v| v.to_string()),
            Some(VERSION.to_string())
        );
        self.update_migration_table(current_migration).await.unwrap();
        match self.get(0).await {
            Ok(migration_data) => {
                match migration_data.target_version {
                    Some(target) => {
                        match target {
                            version if version == "0.2.1" => {
                                // Assumed to be 0 -> 0.2.1
                                return self.migrate_0_to_021().await;
                            },
                            version if version == "0.2.2" => {
                                if migration_data.current_version.is_none() {
                                    self.migrate_0_to_021().await.unwrap();
                                }
                                return self.migrate_021_to_022().await;
                            },
                            version if version == "0.2.3" => {
                                if migration_data.current_version.is_none() {
                                    self.migrate_0_to_021().await.unwrap();
                                }
                                if migration_data.current_version == Some("0.2.1".to_string()) {
                                    self.migrate_021_to_022().await.unwrap();
                                }
                                self.migrate_022_to_023().await.unwrap();
                            },
                            version if version == "0.2.4" => {
                                // 0.2.4 migrations
                            },
                            _ => {
                                log::info!("Unknown target version: {}", target);
                            }
                        }
                    },
                    None => {
                        log::info!("No migration record found");
                    }
                }
            },
            Err(_) => {
                log::info!("No migration record found");
            }
        }
        Ok(())
    }

    async fn update_migration_table(&self, migration: Migration) -> RepositoryResult<()> {
        let migration_data: MigrationData = migration.clone().into();
        match self.save(migration_data).await {
            Ok(_) => Ok(()),
            Err(_) => Err(migration.into()),
        }
    }

    // TODO: make this return a `Migration` that we can just .into() for the migration steps.
    async fn start_migration(&self, current_version: &str, target_version: &str) -> RepositoryResult<()> {
        let migration = Migration::new(Some(current_version.to_string()), Some(target_version.to_string()));
        self.update_migration_table(migration).await
    }
    async fn complete_migration(&self, target_version: &str) -> RepositoryResult<()> {
        let migration = Migration::new(Some(target_version.to_string()), Some(target_version.to_string()));
        self.update_migration_table(migration).await
    }

    pub async fn migrate_022_to_023(&self) -> RepositoryResult<()> {
        log::info!("Starting migration record 0.2.2 -> 0.2.3");
        let migration = self.start_migration("0.2.2", "0.2.3").await;
        let migration_value =  Migration::new(Some("0.2.2".to_string()), Some("0.2.3".to_string()));
        match migration {
            Ok(_) => {
                log::info!("Creating Player database");
                match self.player_repository.create_table().await {
                    Ok(_) => {
                        let db = self.db.lock().await.connect().unwrap();
                        let mut stmt = db.prepare("SELECT COUNT(is_admin) from players").await.unwrap();
                        let mut result = stmt.query(params![]).await.unwrap();
                        let admin = match result.next().await.unwrap() {
                            Some(row) => {
                                let count: i64 = row.get(0).unwrap();
                                if count == 0 {
                                    // insert admin user
                                    log::info!("Inserting admin user");
                                    let admin_email = std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| {
                                        "email@example.com".to_string()
                                    });
                                    self.player_repository.create(Some(
                                        player::Model::new(
                                            None,
                                            "Admin".to_string(),
                                            "User".to_string(),
                                            admin_email,
                                            None,
                                            None,
                                            "Address Example".to_string(),
                                            true
                                        )
                                    )).await
                                } else {
                                    Ok(0)
                                }
                            }
                            None => Err(migration_value.into())
                        };
                        match admin {
                            Ok(_) => {
                                self.complete_migration("0.2.3").await
                            },
                            Err(err) => Err(err),
                        }
                    },
                    Err(_) => Err(migration_value.into())
                }
            },
            Err(err) => Err(err),
        }
    }

    pub async fn migrate_021_to_022(&self) -> RepositoryResult<()> {
        log::info!("Starting migration record 0.2.1 -> 0.2.2");
        let migration = self.start_migration("0.2.1", "0.2.2").await;
        let migration_value =  Migration::new(Some("0.2.1".to_string()), Some("0.2.2".to_string()));
        match migration {
            Ok(_) => {
                log::info!("Updating Faeries database");
                let db = self.db.lock().await.connect().unwrap();
                let result = db.execute(
                    "ALTER TABLE faeries DROP COLUMN auth_token",
                    ()
                ).await;
                match result {
                    Ok(_) => {
                        self.complete_migration("0.2.2").await
                    },
                    Err(_) => {
                        log::error!("Error updating Faeries database: {:?}", result);
                        Err(migration_value.into())
                    },
                }
            },
            Err(err) => Err(err),
        }
    }

    // Mark: - Migrations
    pub async fn migrate_0_to_021(&self) -> RepositoryResult<()> {
        log::info!("Creating migrations table");
        let create_table = self.create_table().await;
        match create_table {
            Ok(_) => {
                log::info!("Starting migration record");
                let migration = self.start_migration("0.0.0", "0.2.1").await;
                let migration_value = Migration::new(Some("0.0.0".to_string()), Some("0.2.1".to_string()));
                let migration_error = Err(migration_value.into());
                match migration {
                    Ok(_) => {
                        log::info!("Creating Faeries database");
                        match self.faery_repository.create_table().await {
                            Ok(_) => {
                                self.complete_migration("0.2.1").await
                            },
                            Err(_) => migration_error
                        }
                    },
                    Err(err) => Err(err),
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