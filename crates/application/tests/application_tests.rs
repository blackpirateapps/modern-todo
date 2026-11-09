use std::sync::{Arc, Mutex};
use application::*;
use domain::*;
use time::{Date, Month};

#[derive(Default)]
struct MockTaskRepo {
    tasks: Mutex<Vec<Task>>,
}

impl TaskRepository for MockTaskRepo {
    fn create(&self, task: &Task) -> Result<(), ApplicationError> {
        self.tasks.lock().unwrap().push(task.clone());
        Ok(())
    }

    fn update(&self, task: &Task) -> Result<(), ApplicationError> {
        let mut list = self.tasks.lock().unwrap();
        if let Some(pos) = list.iter().position(|t| t.id == task.id) {
            list[pos] = task.clone();
            Ok(())
        } else {
            Err(ApplicationError::NotFound {
                entity: "Task",
                id: task.id.to_string(),
            })
        }
    }

    fn get_by_id(&self, id: &TaskId) -> Result<Option<Task>, ApplicationError> {
        Ok(self.tasks.lock().unwrap().iter().find(|t| t.id == *id).cloned())
    }

    fn delete(&self, id: &TaskId) -> Result<(), ApplicationError> {
        self.tasks.lock().unwrap().retain(|t| t.id != *id);
        Ok(())
    }

    fn soft_delete(&self, id: &TaskId) -> Result<(), ApplicationError> {
        let mut list = self.tasks.lock().unwrap();
        if let Some(task) = list.iter_mut().find(|t| t.id == *id) {
            task.soft_delete();
        }
        Ok(())
    }

    fn get_all(&self) -> Result<Vec<Task>, ApplicationError> {
        Ok(self.tasks.lock().unwrap().clone())
    }

    fn get_today(&self, date: Date) -> Result<Vec<Task>, ApplicationError> {
        Ok(self
            .tasks
            .lock()
            .unwrap()
            .iter()
            .filter(|t| !t.is_deleted() && (t.due_date == Some(date) || (t.is_overdue(date) && !t.completed)))
            .cloned()
            .collect())
    }

    fn get_upcoming(&self, from_date: Date) -> Result<Vec<Task>, ApplicationError> {
        Ok(self
            .tasks
            .lock()
            .unwrap()
            .iter()
            .filter(|t| !t.is_deleted() && t.due_date.map_or(false, |d| d > from_date))
            .cloned()
            .collect())
    }

    fn get_inbox(&self) -> Result<Vec<Task>, ApplicationError> {
        Ok(self
            .tasks
            .lock()
            .unwrap()
            .iter()
            .filter(|t| !t.is_deleted() && t.list_id.is_none() && !t.completed)
            .cloned()
            .collect())
    }

    fn get_by_list(&self, list_id: &ListId) -> Result<Vec<Task>, ApplicationError> {
        Ok(self
            .tasks
            .lock()
            .unwrap()
            .iter()
            .filter(|t| !t.is_deleted() && t.list_id == Some(*list_id))
            .cloned()
            .collect())
    }

    fn get_subtasks(&self, parent_id: &TaskId) -> Result<Vec<Task>, ApplicationError> {
        Ok(self
            .tasks
            .lock()
            .unwrap()
            .iter()
            .filter(|t| !t.is_deleted() && t.parent_task_id == Some(*parent_id))
            .cloned()
            .collect())
    }

    fn get_completed(&self) -> Result<Vec<Task>, ApplicationError> {
        Ok(self
            .tasks
            .lock()
            .unwrap()
            .iter()
            .filter(|t| !t.is_deleted() && t.completed)
            .cloned()
            .collect())
    }

    fn search(&self, query: &str) -> Result<Vec<Task>, ApplicationError> {
        let q = query.to_lowercase();
        Ok(self
            .tasks
            .lock()
            .unwrap()
            .iter()
            .filter(|t| !t.is_deleted() && t.title.as_str().to_lowercase().contains(&q))
            .cloned()
            .collect())
    }

    fn assign_tag(&self, _task_id: &TaskId, _tag_id: &TagId) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn remove_tag(&self, _task_id: &TaskId, _tag_id: &TagId) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn get_tags_for_task(&self, _task_id: &TaskId) -> Result<Vec<Tag>, ApplicationError> {
        Ok(vec![])
    }
}

#[derive(Default)]
struct MockListRepo {
    lists: Mutex<Vec<List>>,
}

impl ListRepository for MockListRepo {
    fn create(&self, list: &List) -> Result<(), ApplicationError> {
        self.lists.lock().unwrap().push(list.clone());
        Ok(())
    }

    fn update(&self, list: &List) -> Result<(), ApplicationError> {
        let mut all = self.lists.lock().unwrap();
        if let Some(pos) = all.iter().position(|l| l.id == list.id) {
            all[pos] = list.clone();
            Ok(())
        } else {
            Err(ApplicationError::NotFound {
                entity: "List",
                id: list.id.to_string(),
            })
        }
    }

    fn get_by_id(&self, id: &ListId) -> Result<Option<List>, ApplicationError> {
        Ok(self.lists.lock().unwrap().iter().find(|l| l.id == *id).cloned())
    }

    fn delete(&self, id: &ListId) -> Result<(), ApplicationError> {
        self.lists.lock().unwrap().retain(|l| l.id != *id);
        Ok(())
    }

    fn get_all(&self) -> Result<Vec<List>, ApplicationError> {
        Ok(self.lists.lock().unwrap().clone())
    }
}

#[derive(Default)]
struct MockTagRepo {
    tags: Mutex<Vec<Tag>>,
}

impl TagRepository for MockTagRepo {
    fn create(&self, tag: &Tag) -> Result<(), ApplicationError> {
        self.tags.lock().unwrap().push(tag.clone());
        Ok(())
    }

    fn get_by_id(&self, id: &TagId) -> Result<Option<Tag>, ApplicationError> {
        Ok(self.tags.lock().unwrap().iter().find(|t| t.id == *id).cloned())
    }

    fn get_by_name(&self, name: &str) -> Result<Option<Tag>, ApplicationError> {
        Ok(self.tags.lock().unwrap().iter().find(|t| t.name.as_str() == name).cloned())
    }

    fn get_all(&self) -> Result<Vec<Tag>, ApplicationError> {
        Ok(self.tags.lock().unwrap().clone())
    }

    fn delete(&self, id: &TagId) -> Result<(), ApplicationError> {
        self.tags.lock().unwrap().retain(|t| t.id != *id);
        Ok(())
    }
}

#[test]
fn test_task_service_crud_and_queries() {
    let task_repo = Arc::new(MockTaskRepo::default());
    let list_repo = Arc::new(MockListRepo::default());
    let tag_repo = Arc::new(MockTagRepo::default());

    let service = TaskService::new(task_repo, list_repo, tag_repo, None, None);

    // Create task
    let task = service
        .create_task(CreateTaskCommand {
            title: "Write Daymark tests".to_string(),
            description: Some("Comprehensive integration tests".to_string()),
            priority: Priority::High,
            list_id: None,
            due_date: Some(Date::from_calendar_date(2026, Month::November, 10).unwrap()),
            reminder_at: None,
            parent_task_id: None,
        })
        .unwrap();

    assert_eq!(task.title.as_str(), "Write Daymark tests");
    assert_eq!(task.priority, Priority::High);

    // Complete task
    let completed = service.complete_task(CompleteTaskCommand { id: task.id }).unwrap();
    assert!(completed.completed);

    // Reopen task
    let reopened = service.reopen_task(ReopenTaskCommand { id: task.id }).unwrap();
    assert!(!reopened.completed);

    // Search
    let search_res = service.search_tasks("Daymark").unwrap();
    assert_eq!(search_res.len(), 1);

    // Statistics
    let stats = service
        .get_task_statistics(Date::from_calendar_date(2026, Month::November, 9).unwrap())
        .unwrap();
    assert_eq!(stats.total, 1);
    assert_eq!(stats.pending, 1);
    assert_eq!(stats.completed, 0);
}
