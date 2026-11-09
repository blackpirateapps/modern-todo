use thiserror::Error;
use application::ApplicationError;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Entity not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: String },
}

impl From<StorageError> for ApplicationError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::NotFound { entity, id } => ApplicationError::NotFound { entity, id },
            other => ApplicationError::Repository(other.to_string()),
        }
    }
}
