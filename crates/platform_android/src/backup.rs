use application::{ApplicationError, BackupData, BackupPort};

#[derive(Clone, Default)]
pub struct PlatformBackupService;

impl PlatformBackupService {
    pub fn new() -> Self {
        Self
    }
}

impl BackupPort for PlatformBackupService {
    fn serialize_backup(&self, data: &BackupData) -> Result<String, ApplicationError> {
        serde_json::to_string_pretty(data).map_err(|e| ApplicationError::Backup(e.to_string()))
    }

    fn deserialize_backup(&self, json: &str) -> Result<BackupData, ApplicationError> {
        serde_json::from_str(json).map_err(|e| ApplicationError::Backup(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::Task;

    #[test]
    fn test_backup_serialization_roundtrip() {
        let backup_port = PlatformBackupService::new();
        let task = Task::new("Backup item").unwrap();
        let data = BackupData {
            schema_version: 1,
            exported_at: "2026-11-09T12:00:00Z".to_string(),
            tasks: vec![task],
            lists: vec![],
            tags: vec![],
            task_tags: vec![],
        };

        let json = backup_port.serialize_backup(&data).unwrap();
        assert!(json.contains("Backup item"));

        let deserialized = backup_port.deserialize_backup(&json).unwrap();
        assert_eq!(deserialized.schema_version, 1);
        assert_eq!(deserialized.tasks.len(), 1);
        assert_eq!(deserialized.tasks[0].title.as_str(), "Backup item");
    }
}
