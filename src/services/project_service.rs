use crate::{
    infrastructure::{database::DbError, project_repository::ProjectRepository},
    models::project_model::{Project, ProjectStatus},
    User,
};
use std::{
    cmp::Ordering,
    sync::{Arc, Mutex},
};

use super::time_track_service::TimeTrackService;

#[derive(thiserror::Error, Debug)]
pub enum ProjectError {
    #[error("Project not found")]
    NotFound,
    #[error("The user has too many projects")]
    TooManyProjects,
    #[error("Project exists with same name: {0}")]
    ProjectExistsWithSameName(String),
    #[error("Can not delete project, when there is no time_tracking_service")]
    NoTimeTrackService,
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
    time_track_service: Mutex<Option<Arc<TimeTrackService>>>,
}

impl ProjectService {
    pub fn new(
        repository: Arc<ProjectRepository>,
        time_track_service: Option<Arc<TimeTrackService>>,
    ) -> Self {
        ProjectService {
            repository,
            time_track_service: Mutex::new(time_track_service),
        }
    }

    pub fn set_time_track_service(&mut self, time_track_service: Arc<TimeTrackService>) {
        let mut service = self.time_track_service.lock().unwrap();
        *service = Some(time_track_service);
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
            (ProjectStatus::Active, ProjectStatus::Inactive) => Ordering::Less,
            (ProjectStatus::Inactive, ProjectStatus::Active) => Ordering::Greater,
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
        let time_track_service_guard = self.time_track_service.lock().unwrap();

        if time_track_service_guard.is_none() {
            return Err(ProjectError::NoTimeTrackService);
        }

        let time_track_service = time_track_service_guard.as_ref().unwrap();

        // First, execute the time track deletion
        match time_track_service.delete_for_project(project_id).await {
            Ok(_) => (),
            Err(err) => return Err(ProjectError::Unknown(format!("{:#?}", err))),
        }

        // Then, execute the project deletion
        self.repository
            .delete(project_id, &user.name)
            .await
            .map_err(|err| ProjectError::Unknown(format!("{:#?}", err)))?;

        Ok(())
    }
}
