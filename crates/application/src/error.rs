use domain::DomainError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Entity not found: {entity} (id: {id})")]
    NotFound { entity: &'static str, id: String },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Backup error: {0}")]
    Backup(String),
}
