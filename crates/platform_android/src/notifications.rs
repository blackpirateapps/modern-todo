use application::{ApplicationError, NotificationPort};
use domain::TaskId;
use time::OffsetDateTime;

#[derive(Clone, Default)]
pub struct AndroidNotificationService {
    // In Android runtime, this can hold JNIEnv or Android Activity reference
}

impl AndroidNotificationService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Deterministic 32-bit notification ID derived from the UUID
    pub fn notification_id_from_task(task_id: &TaskId) -> i32 {
        let (first, _) = task_id.0.as_u64_pair();
        (first as i32).abs()
    }
}

impl NotificationPort for AndroidNotificationService {
    fn schedule_reminder(
        &self,
        task_id: &TaskId,
        title: &str,
        reminder_at: OffsetDateTime,
    ) -> Result<(), ApplicationError> {
        let notif_id = Self::notification_id_from_task(task_id);
        tracing::info!(
            task_id = %task_id,
            notification_id = notif_id,
            title = title,
            reminder_at = %reminder_at,
            "Scheduling task reminder"
        );

        #[cfg(target_os = "android")]
        {
            // Android AlarmManager / WorkManager scheduling hook
        }

        Ok(())
    }

    fn cancel_reminder(&self, task_id: &TaskId) -> Result<(), ApplicationError> {
        let notif_id = Self::notification_id_from_task(task_id);
        tracing::info!(
            task_id = %task_id,
            notification_id = notif_id,
            "Cancelling task reminder"
        );

        #[cfg(target_os = "android")]
        {
            // Android AlarmManager cancellation hook
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_id_deterministic() {
        let task_id = TaskId::new();
        let id1 = AndroidNotificationService::notification_id_from_task(&task_id);
        let id2 = AndroidNotificationService::notification_id_from_task(&task_id);
        assert_eq!(id1, id2);
    }
}
