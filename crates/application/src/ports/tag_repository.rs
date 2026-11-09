use domain::{Tag, TagId};
use crate::error::ApplicationError;

pub trait TagRepository: Send + Sync {
    fn create(&self, tag: &Tag) -> Result<(), ApplicationError>;
    fn get_by_id(&self, id: &TagId) -> Result<Option<Tag>, ApplicationError>;
    fn get_by_name(&self, name: &str) -> Result<Option<Tag>, ApplicationError>;
    fn get_all(&self) -> Result<Vec<Tag>, ApplicationError>;
    fn delete(&self, id: &TagId) -> Result<(), ApplicationError>;
}
