use domain::{ListId, Priority, TaskId};
use time::{Date, OffsetDateTime};

#[derive(Debug, Clone)]
pub struct CreateTaskCommand {
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub list_id: Option<ListId>,
    pub due_date: Option<Date>,
    pub reminder_at: Option<OffsetDateTime>,
    pub parent_task_id: Option<TaskId>,
}

#[derive(Debug, Clone)]
pub struct UpdateTaskCommand {
    pub id: TaskId,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub list_id: Option<Option<ListId>>,
    pub due_date: Option<Option<Date>>,
    pub reminder_at: Option<Option<OffsetDateTime>>,
}

#[derive(Debug, Clone)]
pub struct CompleteTaskCommand {
    pub id: TaskId,
}

#[derive(Debug, Clone)]
pub struct ReopenTaskCommand {
    pub id: TaskId,
}

#[derive(Debug, Clone)]
pub struct DeleteTaskCommand {
    pub id: TaskId,
    pub hard_delete: bool,
}

#[derive(Debug, Clone)]
pub struct MoveTaskCommand {
    pub id: TaskId,
    pub target_list_id: Option<ListId>,
    pub new_position: i32,
}

#[derive(Debug, Clone)]
pub struct ChangePriorityCommand {
    pub id: TaskId,
    pub priority: Priority,
}

#[derive(Debug, Clone)]
pub struct CreateListCommand {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateListCommand {
    pub id: ListId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeleteListCommand {
    pub id: ListId,
}

#[derive(Debug, Clone)]
pub struct ScheduleReminderCommand {
    pub task_id: TaskId,
    pub reminder_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct CancelReminderCommand {
    pub task_id: TaskId,
}
