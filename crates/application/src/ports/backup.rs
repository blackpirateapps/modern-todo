use domain::{List, Tag, Task};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::ApplicationError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTagRelation {
    pub task_id: Uuid,
    pub tag_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub schema_version: u32,
    pub exported_at: String,
    pub tasks: Vec<Task>,
    pub lists: Vec<List>,
    pub tags: Vec<Tag>,
    pub task_tags: Vec<TaskTagRelation>,
}

pub trait BackupPort: Send + Sync {
    fn serialize_backup(&self, data: &BackupData) -> Result<String, ApplicationError>;
    fn deserialize_backup(&self, json: &str) -> Result<BackupData, ApplicationError>;
}
