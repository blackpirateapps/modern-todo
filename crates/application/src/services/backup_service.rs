use std::sync::Arc;
use time::OffsetDateTime;

use crate::error::ApplicationError;
use crate::ports::{BackupData, ListRepository, TagRepository, TaskRepository, TaskTagRelation};

pub const CURRENT_BACKUP_SCHEMA_VERSION: u32 = 1;

pub struct BackupService {
    task_repo: Arc<dyn TaskRepository>,
    list_repo: Arc<dyn ListRepository>,
    tag_repo: Arc<dyn TagRepository>,
}

impl BackupService {
    pub fn new(
        task_repo: Arc<dyn TaskRepository>,
        list_repo: Arc<dyn ListRepository>,
        tag_repo: Arc<dyn TagRepository>,
    ) -> Self {
        Self {
            task_repo,
            list_repo,
            tag_repo,
        }
    }

    pub fn export_backup(&self) -> Result<BackupData, ApplicationError> {
        let tasks = self.task_repo.get_all()?;
        let lists = self.list_repo.get_all()?;
        let tags = self.tag_repo.get_all()?;

        let mut relations = Vec::new();
        for task in &tasks {
            let task_tags = self.task_repo.get_tags_for_task(&task.id)?;
            for tag in task_tags {
                relations.push(TaskTagRelation {
                    task_id: task.id.0,
                    tag_id: tag.id.0,
                });
            }
        }

        Ok(BackupData {
            schema_version: CURRENT_BACKUP_SCHEMA_VERSION,
            exported_at: OffsetDateTime::now_utc().to_string(),
            tasks,
            lists,
            tags,
            task_tags: relations,
        })
    }

    pub fn validate_and_import(&self, data: BackupData) -> Result<(), ApplicationError> {
        if data.schema_version != CURRENT_BACKUP_SCHEMA_VERSION {
            return Err(ApplicationError::Backup(format!(
                "Unsupported backup schema version: {}. Expected {}",
                data.schema_version, CURRENT_BACKUP_SCHEMA_VERSION
            )));
        }

        // Import lists
        for list in data.lists {
            if self.list_repo.get_by_id(&list.id)?.is_none() {
                self.list_repo.create(&list)?;
            } else {
                self.list_repo.update(&list)?;
            }
        }

        // Import tags
        for tag in data.tags {
            if self.tag_repo.get_by_id(&tag.id)?.is_none() {
                self.tag_repo.create(&tag)?;
            }
        }

        // Import tasks
        for task in data.tasks {
            if self.task_repo.get_by_id(&task.id)?.is_none() {
                self.task_repo.create(&task)?;
            } else {
                self.task_repo.update(&task)?;
            }
        }

        Ok(())
    }
}
