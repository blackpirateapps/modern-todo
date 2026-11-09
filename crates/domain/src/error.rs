use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum DomainError {
    #[error("Task title cannot be empty")]
    EmptyTaskTitle,

    #[error("Task title exceeds maximum length of {max} characters (got {actual})")]
    TaskTitleTooLong { max: usize, actual: usize },

    #[error("List name cannot be empty")]
    EmptyListName,

    #[error("List name exceeds maximum length of {max} characters (got {actual})")]
    ListNameTooLong { max: usize, actual: usize },

    #[error("Tag name cannot be empty")]
    EmptyTagName,

    #[error("Tag name exceeds maximum length of {max} characters (got {actual})")]
    TagNameTooLong { max: usize, actual: usize },

    #[error("Subtask nesting depth exceeded: maximum allowed depth is 1")]
    SubtaskNestingDepthExceeded,

    #[error("Cannot set a task as its own parent")]
    SelfParentingNotAllowed,

    #[error("Invalid priority level: '{0}'")]
    InvalidPriority(String),

    #[error("Invalid date format or value")]
    InvalidDate,

    #[error("Entity not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: String },
}
