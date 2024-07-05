use crate::models::project_model::{Project, ProjectStatus};
use chrono::{DateTime, Utc};
use rocket::serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(crate = "rocket::serde")]
pub struct NewProjectDto {
    #[validate(length(
        min = 1,
        max = 25,
        message = "must be between 1 and 25 characters long"
    ))]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDto {
    pub project_id: String,
    pub name: String,
    pub status: ProjectStatus,
    pub total_in_seconds: i64,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
}

impl From<Project> for ProjectDto {
    fn from(p: Project) -> Self {
        ProjectDto {
            project_id: p.id,
            name: p.name,
            status: p.status,
            total_in_seconds: p.total_in_seconds,
            created_at: p.created_at,
            modified_at: p.modified_at,
        }
    }
}
