use axum::extract::rejection::JsonRejection;
use serde::Serialize;

// TODO: move
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub enum RepositoryError {
    NotFound,
    AlreadyExists,
    InvalidModel,
    Other,
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[shuttle_runtime::async_trait]
pub trait Repository: Sized + Send + Sync {
    type Item: Serialize + Sized + Send + Sync;
    async fn create(&self, template_item: Option<Self::Item>) -> RepositoryResult<()>;
    async fn save(&self, item: Self::Item) -> RepositoryResult<()>;
    async fn create_table(&self) -> RepositoryResult<()>;
    async fn drop_table(&self) -> RepositoryResult<()>;
    async fn get(&self, id: u32) -> RepositoryResult<Self::Item>;
    async fn get_all(&self) -> RepositoryResult<Vec<Self::Item>>;
    fn table_name() -> String where Self: Sized;
}

impl From<JsonRejection> for RepositoryError {
    fn from(err: JsonRejection) -> Self {
        match err {
            JsonRejection::JsonDataError(_) => RepositoryError::InvalidModel,
            JsonRejection::JsonSyntaxError(_) => RepositoryError::InvalidModel,
            _ => RepositoryError::Other
        }
    }
}