use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{self, DateTime, Utc};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn from_str(s: &str) -> Option<Self> {
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
    pub total_in_seconds: i32,
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

impl From<&HashMap<String, AttributeValue>> for Project {
    fn from(map: &HashMap<String, AttributeValue>) -> Self {
        let id = map
            .get("id")
            .and_then(|v| v.as_s().ok())
            .unwrap()
            .to_string();
        let name = map
            .get("name")
            .and_then(|v| v.as_s().ok())
            .unwrap()
            .to_string();
        let status_str = map.get("status").and_then(|v| v.as_s().ok()).unwrap();
        let status = ProjectStatus::from_str(&status_str).unwrap_or(ProjectStatus::INACTIVE);
        let total_in_seconds = map
            .get("total_in_seconds")
            .and_then(|v| v.as_n().ok())
            .unwrap()
            .parse()
            .unwrap_or(0);
        let created_at = map
            .get("created_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
            .expect("Couldnt parse created_at");
        let created_by = map
            .get("created_by")
            .and_then(|v| v.as_s().ok())
            .unwrap()
            .to_string();
        let modified_at = map
            .get("modified_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse::<DateTime<Utc>>().ok());
        let modified_by = map
            .get("modified_by")
            .and_then(|v| v.as_s().ok())
            .map(|s| s.to_string());

        Project {
            id,
            name,
            status,
            total_in_seconds,
            created_at,
            created_by,
            modified_at,
            modified_by,
        }
    }
}
