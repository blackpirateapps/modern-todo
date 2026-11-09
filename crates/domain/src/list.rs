use serde::{Deserialize, Serialize};
use std::fmt;
use time::OffsetDateTime;
use uuid::Uuid;
use crate::error::DomainError;

pub const MAX_LIST_NAME_LENGTH: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ListId(pub Uuid);

impl ListId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for ListId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ListId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ListName(String);

impl ListName {
    pub fn new(name: impl Into<String>) -> Result<Self, DomainError> {
        let name = name.into();
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyListName);
        }
        if trimmed.chars().count() > MAX_LIST_NAME_LENGTH {
            return Err(DomainError::ListNameTooLong {
                max: MAX_LIST_NAME_LENGTH,
                actual: trimmed.chars().count(),
            });
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ListName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct List {
    pub id: ListId,
    pub name: ListName,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub position: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl List {
    pub fn new(name: impl Into<String>) -> Result<Self, DomainError> {
        let now = OffsetDateTime::now_utc();
        Ok(Self {
            id: ListId::new(),
            name: ListName::new(name)?,
            description: None,
            icon: None,
            color: None,
            position: 0,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn with_details(
        name: impl Into<String>,
        description: Option<String>,
        icon: Option<String>,
        color: Option<String>,
        position: i32,
    ) -> Result<Self, DomainError> {
        let now = OffsetDateTime::now_utc();
        Ok(Self {
            id: ListId::new(),
            name: ListName::new(name)?,
            description,
            icon,
            color,
            position,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn rename(&mut self, new_name: impl Into<String>) -> Result<(), DomainError> {
        self.name = ListName::new(new_name)?;
        self.updated_at = OffsetDateTime::now_utc();
        Ok(())
    }
}
