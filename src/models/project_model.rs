use chrono::{self, DateTime, Utc};
use dynomite::Item;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, dynomite::Attribute)]
pub enum ProjectStatus {
    ACTIVE,
    INACTIVE
}

#[derive(Debug, Clone, Serialize, Deserialize, Item)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub status: ProjectStatus,
    pub total_in_seconds: i32,
    pub created_at: DateTime<Utc>,
    #[dynomite(partition_key)]
    pub created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_by: Option<String>,
}

#[derive(Debug)]
pub enum EProject {
    Active(Project<Active>),
    Inactive(Project<Inactive>),
}

impl From<Project<Active>> for EProject {
    fn from(project: Project<Active>) -> Self {
        EProject::Active(project)
    }
}

impl From<Project<Inactive>> for EProject {
    fn from(project: Project<Inactive>) -> Self {
        EProject::Inactive(project)
    }
}

impl From<&Project<Inactive>> for EProject {
    fn from(project: &Project<Inactive>) -> Self {
        EProject::Inactive(project.clone())
    }
}

impl From<&Project<Active>> for EProject {
    fn from(project: &Project<Active>) -> Self {
        EProject::Active(project.clone())
    }
}
