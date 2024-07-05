use chrono::{self, DateTime, Utc};
use core::fmt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectStatus {
    ACTIVE,
    INACTIVE,
}

impl fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProjectStatus::ACTIVE => write!(f, "ACTIVE"),
            ProjectStatus::INACTIVE => write!(f, "INACTIVE"),
        }
    }
}

impl ProjectStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ACTIVE" => Some(ProjectStatus::ACTIVE),
            "INACTIVE" => Some(ProjectStatus::INACTIVE),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub status: ProjectStatus,
    pub total_in_seconds: i64,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<String>,
}

impl Project {
    pub fn new(project_name: &str, created_by: &str) -> Self {
        Project {
            id: Uuid::new_v4().to_string(),
            name: project_name.to_string(),
            status: ProjectStatus::INACTIVE,
            total_in_seconds: 0,
            created_at: Utc::now(),
            created_by: created_by.to_string(),
            modified_at: None,
            modified_by: None,
        }
    }
}
