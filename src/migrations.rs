use std::sync::Arc;
use libsql::{Connection, Database, params};
use serde::{Deserialize, Serialize};
use crate::version::VERSION;
use semver::{Version, VersionReq};
use tokio::sync::Mutex;
use crate::faery::FaeryRepository;
use crate::player;
use crate::faery;
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

    async fn new_install_check(&mut self) -> bool {
        if self.current_version.is_none() && (self.target_version.to_string() == VERSION.to_string()) {
            self.current_version = Some(Version::parse("0.0.0").unwrap());
            return true
        }
        return false
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
    db: Arc<Mutex<Connection>>,
    player_repository: Arc<PlayerRepository>,
    faery_repository: Arc<FaeryRepository>,
}

impl Manager {
    pub fn new(db: Arc<Mutex<Connection>>, player_repository: Arc<PlayerRepository>, faery_repository: Arc<FaeryRepository>) -> Manager {
        Manager {
            db,
            player_repository,
            faery_repository
        }
    }

    async fn create_tables(&self) -> RepositoryResult<()> {
        let migration = self.create_table().await;
        let player = self.player_repository.create_table().await;
        let faery = self.faery_repository.create_table().await;
        match migration {
            Ok(_) => {
                log::debug!("Migration table created");
                match player {
                    Ok(_) => {
                        log::debug!("Player table created");
                        match faery {
                            Ok(_) => {
                                log::debug!("Faery table created");
                                Ok(())
                            },
                            Err(err) => Err(err),
                        }
                    },
                    Err(err) => Err(err),
                }
            },
            Err(err) => Err(err),
        }
    }

    pub async fn needs_migration(&self) -> bool {
        log::debug!("Checking for migrations");
        let migration_data = self.get(0).await.unwrap_or_else(|_| Migration::new(None, None).into());
        log::debug!("Migration data: {:?}", migration_data);
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
        let mut current_state: Migration = self.get(0).await.unwrap_or_else(|_| Migration::new(None, None).into()).into();
        // TODO: remove the MigrationData / Migration split
        if current_state.new_install_check().await {
            log::info!("New installation detected. Running initial table creation.");
            return self.create_tables().await;
        }
        log::info!("Migrating to {}", VERSION.to_string());
        let current_migration: Migration = Migration::new(
            current_state.current_version.clone().map(|v| v.to_string()),
            Some(VERSION.to_string())
        );
        self.update_migration_table(current_migration).await.unwrap();
        match self.get(0).await {
            Ok(migration_data) => {
                if let Some(target) = migration_data.target_version.clone() {
                    match target.clone() {
                        version if version == "0.2.1" => {
                            // Assumed to be 0 -> 0.2.1
                            return self.migrate_0_to_021(None).await;
                        },
                        version if version == "0.2.2" => {
                            if let Some(current_version) = migration_data.current_version {
                                match current_version.as_str() {
                                    "0.0.0" => {
                                        log::info!("running new installation migration for 0.2.2");
                                        self.migrate_0_to_021(Some(target.clone())).await.unwrap();
                                        return self.complete_migration("0.2.2").await;
                                    },
                                    "0.2.1" => {
                                        log::info!("migrating from 0.2.1 -> 0.2.2");
                                        return self.migrate_021_to_022().await;
                                    }
                                    _ => {}
                                }
                            } else {
                                log::info!("running new installation migration for 0.2.2");
                                self.migrate_0_to_021(Some(target.clone())).await.unwrap();
                                return self.complete_migration("0.2.2").await;
                            }
                        },
                        version if version == "0.2.3" => {
                            if let Some(current_version) = migration_data.current_version {
                                match current_version.as_str() {
                                    "0.0.0" => {
                                        log::info!("running new installation migration");
                                        self.migrate_0_to_021(Some(target.clone())).await.unwrap();
                                        return self.migrate_022_to_023().await;
                                    },
                                    "0.2.1" => {
                                        log::info!("migrating from 0.2.1");
                                        self.migrate_021_to_022().await.unwrap();
                                    },
                                    "0.2.2" => {
                                        log::info!("migrating from 0.2.2");
                                        self.migrate_022_to_023().await.unwrap();
                                    },
                                    _ => { }
                                }
                            } else {
                                self.migrate_0_to_021(Some(target.clone())).await.unwrap();
                                return self.migrate_022_to_023().await;
                            }
                        },
                        version if version == "0.2.4" => {
                            // 0.2.4 migrations
                        },
                        _ => {
                            log::info!("Unknown target version: {}", target);
                        }
                    }
                }
            },
            Err(err) => {
                log::error!("No migration record found: {:?}", err);
            }
        }
        Ok(())
    }

    // TODO: Remove Migration / MigrationData split
    async fn update_migration_table(&self, migration: Migration) -> RepositoryResult<()> {
        let migration_data: MigrationData = migration.clone().into();
        match self.save(migration_data).await {
            Ok(_) => {
                Ok(())
            },
            Err(err) => {
                log::error!("Error updating migration table: {:?}", err);
                log::debug!("Migration: {:?}", migration);
                Err(migration.into())
            },
        }
    }

    // TODO: Remove Migration / MigrationData split
    async fn start_migration(&self, current_version: &str, target_version: &str) -> RepositoryResult<()> {
        let migration = Migration::new(Some(current_version.to_string()), Some(target_version.to_string()));
        self.update_migration_table(migration).await
    }
    async fn complete_migration(&self, target_version: &str) -> RepositoryResult<()> {
        let migration = Migration::new(Some(target_version.to_string()), Some(target_version.to_string()));
        log::info!("Saving completed migration status: {:?}", migration);
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
                        let admin_count = self.player_repository.admin_count().await.unwrap_or(0);
                        if admin_count == 0 {
                            // insert admin user
                            log::info!("Inserting admin user");
                            let admin_email = std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| {
                                "email@example.com".to_string()
                            });
                            match self.player_repository.create(Some(
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
                            )).await {
                                Ok(_) => {
                                    log::info!("completing migration for 0.2.2 -> 0.2.3");
                                    self.complete_migration("0.2.3").await
                                },
                                Err(err) => Err(err),
                            }
                        } else {
                            Ok(())
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
                let db = self.db.lock().await;
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
    pub async fn migrate_0_to_021(&self, completion_target: Option<String>) -> RepositoryResult<()> {
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
                                if let Some(target) = completion_target {
                                    return self.complete_migration(&target).await;
                                }
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
        let db = self.db.lock().await;
        log::info!("Saving migration: {:?}", item);
        let insert_item = item.clone();
        let result = db.execute(
            "UPDATE migrations SET current_version = ?1, target_version = ?2 WHERE id = 0",
            (insert_item.current_version, insert_item.target_version)
        ).await;
        match result {
            Ok(res) => {
                if res == 0 {
                    let update_item = item.clone();
                    match db.execute(
                        "INSERT INTO migrations (id, current_version, target_version) VALUES (0, ?1, ?2)",
                        (update_item.current_version, update_item.target_version)
                    ).await {
                        Ok(_) => Ok(0),
                        Err(err) => {
                            log::error!("Error inserting migration row: {:?}", err);
                            Err(RepositoryError::Other)
                        }
                    }
                } else {
                    Ok(0)
                }
            },
            Err(err) => {
                // Try update
                log::error!("Trying to recover from error by inserting a new migration row: {:?}", err);
                let update_item = item.clone();
                let result = db.execute(
                    "INSERT INTO migrations (id, current_version, target_version) VALUES (0, ?1, ?2)",
                    (update_item.current_version, update_item.target_version)
                ).await;
                match result {
                    Ok(_) => {
                        log::info!("Recovered from error.");
                        Ok(0)
                    },
                    Err(err) => {
                        log::error!("Error trying to recover: {:?}", err);
                        Err(RepositoryError::Other)
                    },
                }
            },
        }
    }

    async fn get(&self, _: i64) -> RepositoryResult<MigrationData> {
        let db = self.db.lock().await;
        let result = db.query(
            "SELECT current_version, target_version FROM migrations WHERE id = 0",
            ()
        ).await;
        match result {
            Ok(mut rows) => {
                let result = rows.next();
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
        let db = self.db.lock().await;
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