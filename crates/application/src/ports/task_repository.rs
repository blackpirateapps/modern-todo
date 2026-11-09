use domain::{ListId, Tag, TagId, Task, TaskId};
use time::Date;
use crate::error::ApplicationError;

pub trait TaskRepository: Send + Sync {
    fn create(&self, task: &Task) -> Result<(), ApplicationError>;
    fn update(&self, task: &Task) -> Result<(), ApplicationError>;
    fn get_by_id(&self, id: &TaskId) -> Result<Option<Task>, ApplicationError>;
    fn delete(&self, id: &TaskId) -> Result<(), ApplicationError>;
    fn soft_delete(&self, id: &TaskId) -> Result<(), ApplicationError>;
    fn get_all(&self) -> Result<Vec<Task>, ApplicationError>;
    fn get_today(&self, date: Date) -> Result<Vec<Task>, ApplicationError>;
    fn get_upcoming(&self, from_date: Date) -> Result<Vec<Task>, ApplicationError>;
    fn get_inbox(&self) -> Result<Vec<Task>, ApplicationError>;
    fn get_by_list(&self, list_id: &ListId) -> Result<Vec<Task>, ApplicationError>;
    fn get_subtasks(&self, parent_id: &TaskId) -> Result<Vec<Task>, ApplicationError>;
    fn get_completed(&self) -> Result<Vec<Task>, ApplicationError>;
    fn search(&self, query: &str) -> Result<Vec<Task>, ApplicationError>;

    fn assign_tag(&self, task_id: &TaskId, tag_id: &TagId) -> Result<(), ApplicationError>;
    fn remove_tag(&self, task_id: &TaskId, tag_id: &TagId) -> Result<(), ApplicationError>;
    fn get_tags_for_task(&self, task_id: &TaskId) -> Result<Vec<Tag>, ApplicationError>;
}
