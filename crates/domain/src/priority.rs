use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use crate::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    #[default]
    None = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::None => "none",
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            Priority::None => "None",
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
        }
    }

    /// Cupertino-styled hex color
    pub fn hex_color(&self) -> &'static str {
        match self {
            Priority::None => "#8E8E93",   // System Gray
            Priority::Low => "#007AFF",    // System Blue
            Priority::Medium => "#FF9500", // System Orange
            Priority::High => "#FF3B30",   // System Red
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Priority {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "none" | "0" | "" => Ok(Priority::None),
            "low" | "1" => Ok(Priority::Low),
            "medium" | "med" | "2" => Ok(Priority::Medium),
            "high" | "3" => Ok(Priority::High),
            other => Err(DomainError::InvalidPriority(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::None < Priority::Low);
        assert!(Priority::Low < Priority::Medium);
        assert!(Priority::Medium < Priority::High);
    }

    #[test]
    fn test_priority_from_str() {
        assert_eq!(Priority::from_str("none").unwrap(), Priority::None);
        assert_eq!(Priority::from_str("low").unwrap(), Priority::Low);
        assert_eq!(Priority::from_str("medium").unwrap(), Priority::Medium);
        assert_eq!(Priority::from_str("high").unwrap(), Priority::High);
        assert!(Priority::from_str("critical").is_err());
    }
}
