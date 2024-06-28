use crate::models::project_model::{Active, EProject, Inactive, Project};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum ProjectStatus {
    ACTIVE,
    INACTIVE,
}

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
    status: ProjectStatus,
    pub total_in_seconds: i32,
    pub created_at: String,
    pub created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_by: Option<String>,
}

impl From<Project<Active>> for ProjectDto {
    fn from(project: Project<Active>) -> Self {
        ProjectDto {
            project_id: project.id,
            name: project.name,
            status: ProjectStatus::ACTIVE,
            total_in_seconds: project.total_in_seconds,
            created_at: project.created_at.to_string(),
            created_by: project.created_by,
            modified_at: project.modified_at.map(|m| m.to_string()),
            modified_by: project.modified_by,
        }
    }
}

impl From<Project<Inactive>> for ProjectDto {
    fn from(project: Project<Inactive>) -> Self {
        ProjectDto {
            project_id: project.id,
            name: project.name,
            status: ProjectStatus::INACTIVE,
            total_in_seconds: project.total_in_seconds,
            created_at: project.created_at.to_string(),
            created_by: project.created_by,
            modified_at: project.modified_at.map(|m| m.to_string()),
            modified_by: project.modified_by,
        }
    }
}

impl From<EProject> for ProjectDto {
    fn from(project: EProject) -> Self {
        match project {
            EProject::Active(project) => project.into(),
            EProject::Inactive(project) => project.into(),
        }
    }
}
