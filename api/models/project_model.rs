use chrono::{self, DateTime, Utc};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, time::Duration};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectStatus {
    #[serde(rename = "ACTIVE")]
    Active,
    #[serde(rename = "INACTIVE")]
    Inactive,
}

impl fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProjectStatus::Active => write!(f, "ACTIVE"),
            ProjectStatus::Inactive => write!(f, "INACTIVE"),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseProjectStatusError {
    #[error("Invalid project status")]
    InvalidStatus,
}

impl FromStr for ProjectStatus {
    type Err = ParseProjectStatusError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACTIVE" => Ok(ProjectStatus::Active),
            "INACTIVE" => Ok(ProjectStatus::Inactive),
            _ => Err(ParseProjectStatusError::InvalidStatus),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub status: ProjectStatus,
    pub total_duration: Duration,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<String>,
}

impl Project {
    pub fn new(project_name: String, created_by: &str) -> Self {
        Project {
            id: Uuid::new_v4().to_string(),
            name: project_name,
            status: ProjectStatus::Inactive,
            total_duration: Duration::new(0, 0),
            created_at: Utc::now(),
            created_by: created_by.to_string(),
            modified_at: None,
            modified_by: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_project_status_display() {
        assert_eq!(
            ProjectStatus::Active.to_string(),
            "ACTIVE",
            "Expected ACTIVE but got {}",
            ProjectStatus::Active
        );
        assert_eq!(
            ProjectStatus::Inactive.to_string(),
            "INACTIVE",
            "Expected INACTIVE but got {}",
            ProjectStatus::Inactive
        );
    }

    #[test]
    fn test_project_status_from_str() {
        assert_eq!(
            ProjectStatus::from_str("ACTIVE").unwrap(),
            ProjectStatus::Active,
            "Expected ProjectStatus::Active but got {:?}",
            ProjectStatus::from_str("ACTIVE")
        );
        assert_eq!(
            ProjectStatus::from_str("INACTIVE").unwrap(),
            ProjectStatus::Inactive,
            "Expected ProjectStatus::Inactive but got {:?}",
            ProjectStatus::from_str("INACTIVE")
        );
        assert!(
            ProjectStatus::from_str("UNKNOWN").is_err(),
            "Expected error for unrecognized status but got {:?}",
            ProjectStatus::from_str("UNKNOWN")
        );
    }

    #[test]
    fn test_project_creation() {
        let project_name = "Test Project";
        let created_by = "Test User";
        let project = Project::new(project_name.to_string(), created_by);

        assert!(Uuid::from_str(&project.id).is_ok());
        assert_eq!(
            project.name, project_name,
            "Expected project name to be '{}' but got '{}'",
            project_name, project.name
        );
        assert_eq!(
            project.created_by, created_by,
            "Expected created_by to be '{}' but got '{}'",
            created_by, project.created_by
        );
        assert_eq!(
            project.status,
            ProjectStatus::Inactive,
            "Expected status to be 'Inactive' but got {:?}",
            project.status
        );
        assert_eq!(
            project.total_duration,
            Duration::new(0, 0),
            "Expected total_duration to be '0' but got {:?}",
            project.total_duration
        );
        assert!(
            project.modified_at.is_none(),
            "Expected modified_at to be 'None' but got {:?}",
            project.modified_at
        );
        assert!(
            project.modified_by.is_none(),
            "Expected modified_by to be 'None' but got {:?}",
            project.modified_by
        );
    }
}
