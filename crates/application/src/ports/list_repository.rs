use domain::{List, ListId};
use crate::error::ApplicationError;

pub trait ListRepository: Send + Sync {
    fn create(&self, list: &List) -> Result<(), ApplicationError>;
    fn update(&self, list: &List) -> Result<(), ApplicationError>;
    fn get_by_id(&self, id: &ListId) -> Result<Option<List>, ApplicationError>;
    fn delete(&self, id: &ListId) -> Result<(), ApplicationError>;
    fn get_all(&self) -> Result<Vec<List>, ApplicationError>;
}
