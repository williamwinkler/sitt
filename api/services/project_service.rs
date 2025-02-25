use tokio::sync::RwLock;

use super::time_track_service::{TimeTrackError, TimeTrackService};
use crate::{
    infrastructure::{database::DbError, project_repository::ProjectRepository},
    models::{
        project_model::{Project, ProjectStatus},
        user_model::{User, UserRole},
    },
};
use std::env;
use std::{cmp::Ordering, sync::Arc};

#[derive(thiserror::Error, Debug)]
pub enum ProjectError {
    #[error("Project not found")]
    NotFound,
    #[error("The user has too many projects")]
    TooManyProjects,
    #[error("Project exists with same name: {0}")]
    ProjectExistsWithSameName(String),
    #[error("Can not delete project, when time_tracking_service is None")]
    NoTimeTrackService,
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error("TimeTrack erro: {0}")]
    TimeTrackError(TimeTrackError)
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

impl From<TimeTrackError> for ProjectError {
    fn from(error: TimeTrackError) -> Self {
        ProjectError::TimeTrackError(error)
    }
}

#[derive(Debug)]
pub struct ProjectService {
    repository: Arc<ProjectRepository>,
    time_track_service: RwLock<Option<Arc<TimeTrackService>>>,
    max_projects: usize,
}

impl ProjectService {
    pub fn new(
        repository: Arc<ProjectRepository>,
        time_track_service: Option<Arc<TimeTrackService>>,
    ) -> Self {
        // Get max number of projects from env
        let max_projects = env::var("MAX_PROJECTS")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(15);

        ProjectService {
            repository,
            time_track_service: RwLock::new(time_track_service),
            max_projects,
        }
    }

    pub async fn set_time_track_service(&self, time_track_service: Arc<TimeTrackService>) {
        let mut service = self.time_track_service.write().await;
        *service = Some(time_track_service);
    }

    pub async fn create(&self, user: &User, project_name: String) -> Result<Project, ProjectError> {
        // Get existing projects for user
        let existing_projects = self.repository.get_all(user).await?;

        // Each user can maximum have 15 projects (except admins)
        if user.role != UserRole::Admin && existing_projects.len() >= self.max_projects {
            return Err(ProjectError::TooManyProjects);
        }

        // Check if there already is a project with the same name
        if existing_projects.iter().any(|p| p.name == project_name) {
            return Err(ProjectError::ProjectExistsWithSameName(project_name));
        }

        // Create the new project
        let project = Project::new(project_name, &user.id);
        self.repository.create(&project).await?;

        Ok(project)
    }

    pub async fn get_all(&self, user: &User) -> Result<Vec<Project>, ProjectError> {
        let mut projects = self.repository.get_all(user).await?;

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

        let has_active_project = projects.iter().any(|p| p.status == ProjectStatus::Active);

        if has_active_project {
            // Aquire read lock...
            let time_track_service_guard = self.time_track_service.read().await;
            // Check if the time_track_service is set
            let time_track_service = match time_track_service_guard.as_ref() {
                Some(service) => service,
                None => return Err(ProjectError::NoTimeTrackService),
            };

            // Map projects, updating the total_duration for ACTIVE projects
            for project in &mut projects {
                if project.status == ProjectStatus::Active {
                    let time_track = time_track_service
                        .get_in_progress(user, &project.id, &project.name)
                        .await?;

                    project.total_duration += time_track.total_duration;
                }
            }
        }

        Ok(projects)
    }

    pub async fn get(&self, user: &User, project_id: &str) -> Result<Project, ProjectError> {
        let mut project = self.repository.get(user, project_id).await?;

        // If project is active, get up to date duration
        if project.status == ProjectStatus::Active {
            // Aquire read lock...
            let time_track_service_guard = self.time_track_service.read().await;
            // Check if the time_track_service is set
            let time_track_service = match time_track_service_guard.as_ref() {
                Some(service) => service,
                None => return Err(ProjectError::NoTimeTrackService),
            };

            // There should be a time track object in the DB for the project with status IN_PROGRESS
            let time_track = time_track_service
                .get_in_progress(user, &project.id, &project.name)
                .await?;

            project.total_duration += time_track.total_duration;
        }

        Ok(project)
    }

    pub async fn update_name(
        &self,
        user: &User,
        project_id: String,
        new_project_name: String,
    ) -> Result<Project, ProjectError> {
        let mut project = self.repository.get(user, &project_id).await?;

        project.name = new_project_name;

        let updated_project = self.repository.update(user, &mut project).await?;
        Ok(updated_project)
    }

    pub async fn update(
        &self,
        user: &User,
        project: &mut Project,
    ) -> Result<Project, ProjectError> {
        let project = self.repository.update(user, project).await?;

        Ok(project)
    }

    pub async fn delete(&self, user: &User, project_id: &str) -> Result<(), ProjectError> {
        let time_track_service_guard = self.time_track_service.read().await;

        // Check if the time_track_service is set
        let time_track_service = match time_track_service_guard.as_ref() {
            Some(service) => service,
            None => return Err(ProjectError::NoTimeTrackService),
        };

        // First, execute the time track deletion
        match time_track_service
            .delete_for_project(user, project_id)
            .await
        {
            Ok(_) => (),
            Err(err) => match err {
                TimeTrackError::ProjectNotFound => (),
                _ => {
                    return Err(ProjectError::Unknown(format!(
                        "ProjectService.delete() delete time track items: {:#?}",
                        err
                    )))
                }
            },
        }

        // Then, execute the project deletion
        let result = self.repository.delete(user, project_id).await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => match err {
                DbError::NotFound => Err(ProjectError::NotFound),
                _ => Err(ProjectError::Unknown(format!(
                    "ProjectService.delete() delete project: {:#?}",
                    err
                ))),
            },
        }
    }
}
