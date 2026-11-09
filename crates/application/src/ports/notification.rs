use domain::TaskId;
use time::OffsetDateTime;
use crate::error::ApplicationError;

pub trait NotificationPort: Send + Sync {
    fn schedule_reminder(
        &self,
        task_id: &TaskId,
        title: &str,
        reminder_at: OffsetDateTime,
    ) -> Result<(), ApplicationError>;

    fn cancel_reminder(&self, task_id: &TaskId) -> Result<(), ApplicationError>;
}
