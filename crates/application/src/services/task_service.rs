use std::sync::Arc;
use domain::{DomainError, List, ListId, Task, TaskId};
use time::Date;

use crate::commands::*;
use crate::error::ApplicationError;
use crate::ports::{HapticPort, ListRepository, NotificationPort, TagRepository, TaskRepository};
use crate::queries::TaskStatistics;

pub struct TaskService {
    task_repo: Arc<dyn TaskRepository>,
    list_repo: Arc<dyn ListRepository>,
    tag_repo: Arc<dyn TagRepository>,
    notifications: Option<Arc<dyn NotificationPort>>,
    haptics: Option<Arc<dyn HapticPort>>,
}

impl TaskService {
    pub fn new(
        task_repo: Arc<dyn TaskRepository>,
        list_repo: Arc<dyn ListRepository>,
        tag_repo: Arc<dyn TagRepository>,
        notifications: Option<Arc<dyn NotificationPort>>,
        haptics: Option<Arc<dyn HapticPort>>,
    ) -> Self {
        Self {
            task_repo,
            list_repo,
            tag_repo,
            notifications,
            haptics,
        }
    }

    pub fn create_task(&self, cmd: CreateTaskCommand) -> Result<Task, ApplicationError> {
        let mut task = if let Some(parent_id) = cmd.parent_task_id {
            let parent = self
                .task_repo
                .get_by_id(&parent_id)?
                .ok_or_else(|| ApplicationError::NotFound {
                    entity: "Task",
                    id: parent_id.to_string(),
                })?;

            if parent.is_subtask() {
                return Err(ApplicationError::Domain(DomainError::SubtaskNestingDepthExceeded));
            }
            Task::new_subtask(parent_id, cmd.title, false)?
        } else {
            Task::new(cmd.title)?
        };

        task.description = cmd.description;
        task.priority = cmd.priority;
        task.list_id = cmd.list_id;
        task.due_date = cmd.due_date;
        task.reminder_at = cmd.reminder_at;

        self.task_repo.create(&task)?;

        if let (Some(notifications), Some(reminder_at)) = (&self.notifications, task.reminder_at) {
            let _ = notifications.schedule_reminder(&task.id, task.title.as_str(), reminder_at);
        }

        if let Some(haptics) = &self.haptics {
            let _ = haptics.light_feedback();
        }

        Ok(task)
    }

    pub fn update_task(&self, cmd: UpdateTaskCommand) -> Result<Task, ApplicationError> {
        let mut task = self
            .task_repo
            .get_by_id(&cmd.id)?
            .ok_or_else(|| ApplicationError::NotFound {
                entity: "Task",
                id: cmd.id.to_string(),
            })?;

        if let Some(title) = cmd.title {
            task.set_title(title)?;
        }
        if let Some(desc) = cmd.description {
            task.set_description(Some(desc));
        }
        if let Some(priority) = cmd.priority {
            task.set_priority(priority);
        }
        if let Some(list_id) = cmd.list_id {
            task.set_list(list_id);
        }
        if let Some(due_date) = cmd.due_date {
            task.set_due_date(due_date);
        }
        if let Some(reminder_at) = cmd.reminder_at {
            task.set_reminder(reminder_at);
            if let Some(notifications) = &self.notifications {
                if let Some(rem) = reminder_at {
                    let _ = notifications.schedule_reminder(&task.id, task.title.as_str(), rem);
                } else {
                    let _ = notifications.cancel_reminder(&task.id);
                }
            }
        }

        self.task_repo.update(&task)?;
        Ok(task)
    }

    pub fn complete_task(&self, cmd: CompleteTaskCommand) -> Result<Task, ApplicationError> {
        let mut task = self
            .task_repo
            .get_by_id(&cmd.id)?
            .ok_or_else(|| ApplicationError::NotFound {
                entity: "Task",
                id: cmd.id.to_string(),
            })?;

        task.complete();
        self.task_repo.update(&task)?;

        if let Some(notifications) = &self.notifications {
            let _ = notifications.cancel_reminder(&task.id);
        }

        if let Some(haptics) = &self.haptics {
            let _ = haptics.success_feedback();
        }

        Ok(task)
    }

    pub fn reopen_task(&self, cmd: ReopenTaskCommand) -> Result<Task, ApplicationError> {
        let mut task = self
            .task_repo
            .get_by_id(&cmd.id)?
            .ok_or_else(|| ApplicationError::NotFound {
                entity: "Task",
                id: cmd.id.to_string(),
            })?;

        task.reopen();
        self.task_repo.update(&task)?;
        Ok(task)
    }

    pub fn delete_task(&self, cmd: DeleteTaskCommand) -> Result<(), ApplicationError> {
        if let Some(notifications) = &self.notifications {
            let _ = notifications.cancel_reminder(&cmd.id);
        }

        if cmd.hard_delete {
            self.task_repo.delete(&cmd.id)?;
        } else {
            self.task_repo.soft_delete(&cmd.id)?;
        }

        if let Some(haptics) = &self.haptics {
            let _ = haptics.warning_feedback();
        }

        Ok(())
    }

    pub fn move_task(&self, cmd: MoveTaskCommand) -> Result<Task, ApplicationError> {
        let mut task = self
            .task_repo
            .get_by_id(&cmd.id)?
            .ok_or_else(|| ApplicationError::NotFound {
                entity: "Task",
                id: cmd.id.to_string(),
            })?;

        task.set_list(cmd.target_list_id);
        task.set_position(cmd.new_position);
        self.task_repo.update(&task)?;
        Ok(task)
    }

    pub fn change_priority(&self, cmd: ChangePriorityCommand) -> Result<Task, ApplicationError> {
        let mut task = self
            .task_repo
            .get_by_id(&cmd.id)?
            .ok_or_else(|| ApplicationError::NotFound {
                entity: "Task",
                id: cmd.id.to_string(),
            })?;

        task.set_priority(cmd.priority);
        self.task_repo.update(&task)?;
        Ok(task)
    }

    pub fn create_list(&self, cmd: CreateListCommand) -> Result<List, ApplicationError> {
        let list = List::with_details(cmd.name, cmd.description, cmd.icon, cmd.color, 0)?;
        self.list_repo.create(&list)?;
        Ok(list)
    }

    pub fn update_list(&self, cmd: UpdateListCommand) -> Result<List, ApplicationError> {
        let mut list = self
            .list_repo
            .get_by_id(&cmd.id)?
            .ok_or_else(|| ApplicationError::NotFound {
                entity: "List",
                id: cmd.id.to_string(),
            })?;

        if let Some(name) = cmd.name {
            list.rename(name)?;
        }
        if let Some(desc) = cmd.description {
            list.description = Some(desc);
        }
        if let Some(icon) = cmd.icon {
            list.icon = Some(icon);
        }
        if let Some(color) = cmd.color {
            list.color = Some(color);
        }

        self.list_repo.update(&list)?;
        Ok(list)
    }

    pub fn delete_list(&self, cmd: DeleteListCommand) -> Result<(), ApplicationError> {
        self.list_repo.delete(&cmd.id)
    }

    pub fn get_today_tasks(&self, date: Date) -> Result<Vec<Task>, ApplicationError> {
        self.task_repo.get_today(date)
    }

    pub fn get_upcoming_tasks(&self, from_date: Date) -> Result<Vec<Task>, ApplicationError> {
        self.task_repo.get_upcoming(from_date)
    }

    pub fn get_inbox_tasks(&self) -> Result<Vec<Task>, ApplicationError> {
        self.task_repo.get_inbox()
    }

    pub fn get_tasks_by_list(&self, list_id: &ListId) -> Result<Vec<Task>, ApplicationError> {
        self.task_repo.get_by_list(list_id)
    }

    pub fn get_subtasks(&self, parent_id: &TaskId) -> Result<Vec<Task>, ApplicationError> {
        self.task_repo.get_subtasks(parent_id)
    }

    pub fn get_completed_tasks(&self) -> Result<Vec<Task>, ApplicationError> {
        self.task_repo.get_completed()
    }

    pub fn search_tasks(&self, query: &str) -> Result<Vec<Task>, ApplicationError> {
        self.task_repo.search(query)
    }

    pub fn get_all_lists(&self) -> Result<Vec<List>, ApplicationError> {
        self.list_repo.get_all()
    }

    pub fn get_all_tags(&self) -> Result<Vec<domain::Tag>, ApplicationError> {
        self.tag_repo.get_all()
    }

    pub fn create_tag(&self, name: &str, color: Option<String>) -> Result<domain::Tag, ApplicationError> {
        let tag = domain::Tag::new(name, color)?;
        self.tag_repo.create(&tag)?;
        Ok(tag)
    }

    pub fn assign_tag(&self, task_id: &TaskId, tag_id: &domain::TagId) -> Result<(), ApplicationError> {
        self.task_repo.assign_tag(task_id, tag_id)
    }

    pub fn get_tags_for_task(&self, task_id: &TaskId) -> Result<Vec<domain::Tag>, ApplicationError> {
        self.task_repo.get_tags_for_task(task_id)
    }

    pub fn get_task_statistics(&self, current_date: Date) -> Result<TaskStatistics, ApplicationError> {
        let all_tasks = self.task_repo.get_all()?;
        let mut stats = TaskStatistics::default();

        for task in all_tasks {
            if task.is_deleted() {
                continue;
            }
            stats.total += 1;
            if task.completed {
                stats.completed += 1;
            } else {
                stats.pending += 1;
                if task.is_overdue(current_date) {
                    stats.overdue += 1;
                }
                if task.is_due_today(current_date) {
                    stats.due_today += 1;
                }
            }
        }

        Ok(stats)
    }
}
