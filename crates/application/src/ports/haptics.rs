use crate::error::ApplicationError;

pub trait HapticPort: Send + Sync {
    fn light_feedback(&self) -> Result<(), ApplicationError>;
    fn success_feedback(&self) -> Result<(), ApplicationError>;
    fn warning_feedback(&self) -> Result<(), ApplicationError>;
}
