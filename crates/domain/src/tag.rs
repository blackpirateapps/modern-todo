use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use crate::error::DomainError;

pub const MAX_TAG_NAME_LENGTH: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TagId(pub Uuid);

impl TagId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for TagId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TagName(String);

impl TagName {
    pub fn new(name: impl Into<String>) -> Result<Self, DomainError> {
        let name = name.into();
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyTagName);
        }
        if trimmed.chars().count() > MAX_TAG_NAME_LENGTH {
            return Err(DomainError::TagNameTooLong {
                max: MAX_TAG_NAME_LENGTH,
                actual: trimmed.chars().count(),
            });
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TagName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub id: TagId,
    pub name: TagName,
    pub color: Option<String>,
}

impl Tag {
    pub fn new(name: impl Into<String>, color: Option<String>) -> Result<Self, DomainError> {
        Ok(Self {
            id: TagId::new(),
            name: TagName::new(name)?,
            color,
        })
    }
}
