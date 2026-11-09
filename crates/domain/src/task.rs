use serde::{Deserialize, Serialize};
use std::fmt;
use time::{Date, OffsetDateTime};
use uuid::Uuid;
use crate::error::DomainError;
use crate::list::ListId;
use crate::priority::Priority;

pub const MAX_TASK_TITLE_LENGTH: usize = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TaskId(pub Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskTitle(String);

impl TaskTitle {
    pub fn new(title: impl Into<String>) -> Result<Self, DomainError> {
        let title = title.into();
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyTaskTitle);
        }
        if trimmed.chars().count() > MAX_TASK_TITLE_LENGTH {
            return Err(DomainError::TaskTitleTooLong {
                max: MAX_TASK_TITLE_LENGTH,
                actual: trimmed.chars().count(),
            });
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TaskTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub title: TaskTitle,
    pub description: Option<String>,
    pub priority: Priority,
    pub completed: bool,
    pub list_id: Option<ListId>,
    pub due_date: Option<Date>,
    pub reminder_at: Option<OffsetDateTime>,
    pub parent_task_id: Option<TaskId>,
    pub position: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub completed_at: Option<OffsetDateTime>,
    pub deleted_at: Option<OffsetDateTime>,
}

impl Task {
    pub fn new(title: impl Into<String>) -> Result<Self, DomainError> {
        let now = OffsetDateTime::now_utc();
        Ok(Self {
            id: TaskId::new(),
            title: TaskTitle::new(title)?,
            description: None,
            priority: Priority::None,
            completed: false,
            list_id: None,
            due_date: None,
            reminder_at: None,
            parent_task_id: None,
            position: 0,
            created_at: now,
            updated_at: now,
            completed_at: None,
            deleted_at: None,
        })
    }

    pub fn new_subtask(
        parent_id: TaskId,
        title: impl Into<String>,
        parent_is_subtask: bool,
    ) -> Result<Self, DomainError> {
        if parent_is_subtask {
            return Err(DomainError::SubtaskNestingDepthExceeded);
        }
        let mut task = Self::new(title)?;
        task.parent_task_id = Some(parent_id);
        Ok(task)
    }

    pub fn complete(&mut self) {
        if !self.completed {
            self.completed = true;
            let now = OffsetDateTime::now_utc();
            self.completed_at = Some(now);
            self.updated_at = now;
        }
    }

    pub fn reopen(&mut self) {
        if self.completed {
            self.completed = false;
            self.completed_at = None;
            self.updated_at = OffsetDateTime::now_utc();
        }
    }

    pub fn toggle_completed(&mut self) {
        if self.completed {
            self.reopen();
        } else {
            self.complete();
        }
    }

    pub fn soft_delete(&mut self) {
        let now = OffsetDateTime::now_utc();
        self.deleted_at = Some(now);
        self.updated_at = now;
    }

    pub fn restore(&mut self) {
        self.deleted_at = None;
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn is_subtask(&self) -> bool {
        self.parent_task_id.is_some()
    }

    pub fn set_title(&mut self, title: impl Into<String>) -> Result<(), DomainError> {
        self.title = TaskTitle::new(title)?;
        self.updated_at = OffsetDateTime::now_utc();
        Ok(())
    }

    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn set_due_date(&mut self, due_date: Option<Date>) {
        self.due_date = due_date;
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn set_reminder(&mut self, reminder: Option<OffsetDateTime>) {
        self.reminder_at = reminder;
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn set_list(&mut self, list_id: Option<ListId>) {
        self.list_id = list_id;
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn set_position(&mut self, position: i32) {
        self.position = position;
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn is_overdue(&self, current_date: Date) -> bool {
        if self.completed || self.is_deleted() {
            return false;
        }
        match self.due_date {
            Some(due) => due < current_date,
            None => false,
        }
    }

    pub fn is_due_today(&self, current_date: Date) -> bool {
        if self.completed || self.is_deleted() {
            return false;
        }
        match self.due_date {
            Some(due) => due == current_date,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_task() {
        let task = Task::new("Buy milk").unwrap();
        assert_eq!(task.title.as_str(), "Buy milk");
        assert!(!task.completed);
        assert_eq!(task.priority, Priority::None);
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_empty_title_fails() {
        let res = Task::new("   ");
        assert_eq!(res.unwrap_err(), DomainError::EmptyTaskTitle);
    }

    #[test]
    fn test_completion_lifecycle() {
        let mut task = Task::new("Clean room").unwrap();
        task.complete();
        assert!(task.completed);
        assert!(task.completed_at.is_some());

        task.reopen();
        assert!(!task.completed);
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_subtask_nesting_rule() {
        let parent = Task::new("Parent task").unwrap();
        let subtask = Task::new_subtask(parent.id, "Subtask 1", false).unwrap();
        assert!(subtask.is_subtask());

        // A subtask cannot be parent to another subtask
        let invalid = Task::new_subtask(subtask.id, "Nested subtask", true);
        assert_eq!(invalid.unwrap_err(), DomainError::SubtaskNestingDepthExceeded);
    }
}
