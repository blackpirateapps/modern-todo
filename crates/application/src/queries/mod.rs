use domain::ListId;
use serde::{Deserialize, Serialize};
use time::Date;

#[derive(Debug, Clone)]
pub struct GetTodayTasksQuery {
    pub date: Date,
}

#[derive(Debug, Clone)]
pub struct GetUpcomingTasksQuery {
    pub from_date: Date,
}

#[derive(Debug, Clone, Default)]
pub struct GetInboxTasksQuery;

#[derive(Debug, Clone)]
pub struct GetTasksByListQuery {
    pub list_id: ListId,
}

#[derive(Debug, Clone, Default)]
pub struct GetCompletedTasksQuery;

#[derive(Debug, Clone)]
pub struct SearchTasksQuery {
    pub query: String,
}

#[derive(Debug, Clone)]
pub struct GetTaskStatisticsQuery {
    pub current_date: Date,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskStatistics {
    pub total: usize,
    pub completed: usize,
    pub pending: usize,
    pub overdue: usize,
    pub due_today: usize,
}
