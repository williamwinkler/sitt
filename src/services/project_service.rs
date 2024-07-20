use crate::{
    infrastructure::{database::DbError, project_repository::ProjectRepository},
    models::project_model::{Project, ProjectStatus},
    User,
};
use std::{cmp::Ordering, sync::Arc};

#[derive(thiserror::Error, Debug)]
pub enum ProjectError {
    #[error("Project not found")]
    NotFound,
    #[error("The user has too many projects")]
    TooManyProjects,
    #[error("Project exists with same name: {0}")]
    ProjectExistsWithSameName(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<DbError> for ProjectError {
    fn from(error: DbError) -> Self {
        match error {
            DbError::NotFound => ProjectError::NotFound,
            DbError::Convertion { table, id } => ProjectError::Unknown(format!(
                "Conversion error in table '{}' for id '{}'",
                table, id
            )),
            DbError::Unknown(msg) => ProjectError::Unknown(msg),
        }
    }
}

#[derive(Debug)]
pub struct ProjectService {
    repository: Arc<ProjectRepository>,
}

impl ProjectService {
    pub fn new(repository: Arc<ProjectRepository>) -> Self {
        ProjectService { repository }
    }

    pub async fn create(&self, project_name: &str, user: &User) -> Result<(Project), ProjectError> {
        // Get existing projects for user
        let existing_projects = self.repository.get_all(&user.name).await?;

        // Each user can maximum have 15 projects
        if existing_projects.len() >= 15 {
            return Err(ProjectError::TooManyProjects);
        }

        // Check if there already is a project with the same name
        if existing_projects.iter().any(|p| p.name == project_name) {
            return Err(ProjectError::ProjectExistsWithSameName(String::from(
                project_name,
            )));
        }

        // Create the new project
        let project = Project::new(project_name, &user.name);
        self.repository.create(&project).await?;

        Ok(project)
    }

    pub async fn get_all(&self, user: &User) -> Result<Vec<Project>, ProjectError> {
        let mut projects = self.repository.get_all(&user.name).await?;

        // Sort the projects, so the ACTIVE projects occur first in the list
        projects.sort_by(|a, b| match (&a.status, &b.status) {
            (ProjectStatus::ACTIVE, ProjectStatus::INACTIVE) => Ordering::Less,
            (ProjectStatus::INACTIVE, ProjectStatus::ACTIVE) => Ordering::Greater,
            _ => {
                let a_date = a.modified_at.unwrap_or(a.created_at);
                let b_date = b.modified_at.unwrap_or(b.created_at);
                b_date.cmp(&a_date)
            }
        });

        Ok(projects)
    }

    pub async fn get(&self, project_id: &str, user: &User) -> Result<Project, ProjectError> {
        let project = self.repository.get(project_id, &user.name).await?;

        Ok(project)
    }

    pub async fn update(
        &self,
        project: &mut Project,
        user: &User,
    ) -> Result<Project, ProjectError> {
        let project = self.repository.update(project, user).await?;

        Ok(project)
    }

    pub async fn delete(&self, project_id: &str, user: &User) -> Result<(), ProjectError> {
        let result = self.repository.delete(project_id, &user.name).await?;

        Ok(result)
    }
}
