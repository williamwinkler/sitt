use crate::models::project_model::{Project, ProjectStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewProjectDto {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProjectDto {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDto {
    pub project_id: String,
    pub name: String,
    pub status: ProjectStatus,
    pub total_in_seconds: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_by: Option<String>,
}

impl From<Project> for ProjectDto {
    fn from(project: Project) -> Self {
        ProjectDto {
            project_id: project.id,
            name: project.name,
            status: project.status,
            total_in_seconds: project.total_in_seconds,
            created_at: project.created_at,
            created_by: project.created_by,
            modified_at: project.modified_at,
            modified_by: project.modified_by,
        }
    }
}
