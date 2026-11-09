use application::{ListRepository, TagRepository, TaskRepository};
use domain::{Priority, Tag, Task};
use time::{Date, Month};
use storage::{Database, SqliteListRepository, SqliteTagRepository, SqliteTaskRepository};

#[test]
fn test_sqlite_task_crud() {
    let db = Database::in_memory().expect("Failed to create in-memory database");
    let repo = SqliteTaskRepository::new(db);

    let mut task = Task::new("Buy groceries").unwrap();
    task.set_priority(Priority::High);
    let today = Date::from_calendar_date(2026, Month::November, 9).unwrap();
    task.set_due_date(Some(today));

    // Create
    repo.create(&task).expect("Failed to create task");

    // Read
    let fetched = repo.get_by_id(&task.id).expect("Failed to fetch task");
    assert!(fetched.is_some());
    let fetched_task = fetched.unwrap();
    assert_eq!(fetched_task.title.as_str(), "Buy groceries");
    assert_eq!(fetched_task.priority, Priority::High);
    assert_eq!(fetched_task.due_date, Some(today));

    // Update
    task.complete();
    repo.update(&task).expect("Failed to update task");

    let updated = repo.get_by_id(&task.id).unwrap().unwrap();
    assert!(updated.completed);
    assert!(updated.completed_at.is_some());

    // Search
    let results = repo.search("groceries").expect("Failed to search");
    assert_eq!(results.len(), 1);

    // Delete
    repo.delete(&task.id).expect("Failed to delete");
    let after_delete = repo.get_by_id(&task.id).unwrap();
    assert!(after_delete.is_none());
}

#[test]
fn test_sqlite_lists_and_tags() {
    let db = Database::in_memory().expect("Failed to create in-memory database");
    let list_repo = SqliteListRepository::new(db.clone());
    let tag_repo = SqliteTagRepository::new(db.clone());
    let task_repo = SqliteTaskRepository::new(db);

    let list = domain::List::new("Work").unwrap();
    list_repo.create(&list).unwrap();

    let tag = Tag::new("Urgent", Some("#FF0000".to_string())).unwrap();
    tag_repo.create(&tag).unwrap();

    let mut task = Task::new("Deploy application").unwrap();
    task.set_list(Some(list.id));
    task_repo.create(&task).unwrap();

    task_repo.assign_tag(&task.id, &tag.id).unwrap();
    let tags = task_repo.get_tags_for_task(&task.id).unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name.as_str(), "Urgent");

    let by_list = task_repo.get_by_list(&list.id).unwrap();
    assert_eq!(by_list.len(), 1);
}
