use axum::extract::rejection::JsonRejection;
use semver::Version;
use serde::Serialize;

// TODO: move
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub enum RepositoryError {
    NotFound,
    AlreadyExists,
    InvalidModel,
    MigrationFailed(Version, Version),
    Other,
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

pub trait RepositoryItem {
    fn masked_columns(is_admin: bool) -> Vec<String>;
    fn saved_columns() -> Vec<String>;
    fn all_columns() -> Vec<String>;
    fn table_name() -> String where Self: Sized;
}

#[shuttle_runtime::async_trait]
pub trait Repository: Sized + Send + Sync {
    type Item: RepositoryItem + Serialize + Sized + Send + Sync;
    type RowIdentifier: RepositoryRowIdentifier;
    async fn create(&self, template_item: Option<Self::Item>) -> RepositoryResult<Self::RowIdentifier> {
        match template_item {
            Some(template_item) => {
                self.save(template_item).await
            },
            None => Err(RepositoryError::Other),
        }
    }
    async fn save(&self, item: Self::Item) -> RepositoryResult<Self::RowIdentifier>;
    async fn get(&self, id: Self::RowIdentifier) -> RepositoryResult<Self::Item>;
    async fn get_all(&self) -> RepositoryResult<Vec<Self::Item>>;
    async fn delete(&self, id: Self::RowIdentifier) -> RepositoryResult<()>;
    async fn create_table(&self) -> RepositoryResult<()>;
    async fn drop_table(&self) -> RepositoryResult<()>;
    fn table_name() -> String {
        Self::Item::table_name()
    }
}

pub trait RepositoryRowIdentifier {}
impl RepositoryRowIdentifier for u32 {}
impl RepositoryRowIdentifier for i32 {}
impl RepositoryRowIdentifier for u64 {}
impl RepositoryRowIdentifier for i64 {}
impl RepositoryRowIdentifier for () {}

impl From<JsonRejection> for RepositoryError {
    fn from(err: JsonRejection) -> Self {
        match err {
            JsonRejection::JsonDataError(_) => RepositoryError::InvalidModel,
            JsonRejection::JsonSyntaxError(_) => RepositoryError::InvalidModel,
            _ => RepositoryError::Other
        }
    }
}