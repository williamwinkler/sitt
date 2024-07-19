use super::project_service::{ProjectError, ProjectService};
use crate::{
    infrastructure::{database::DbError, time_track_repository::TimeTrackRepository},
    models::{
        project_model::ProjectStatus,
        time_track_model::{TimeTrack, TimeTrackStatus},
    },
    User,
};
use chrono::Utc;
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum TimeTrackError {
    #[error("Time tracking not found")]
    NotFound,
    #[error("Project not found")]
    ProjectNotFound,
    #[error("No time tracking in is progress for project '{0}'.")]
    NoInProgressTimeTracking(String),
    #[error("You are already tracking time on project '{0}'")]
    AlreadyTrackingTime(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<DbError> for TimeTrackError {
    fn from(error: DbError) -> Self {
        match error {
            DbError::NotFound => TimeTrackError::NotFound,
            DbError::AlreadyExists { key, value } => TimeTrackError::Unknown(format!(
                "Key '{}' already exists with value '{}'",
                key, value
            )),
            DbError::Convertion { table, id } => TimeTrackError::Unknown(format!(
                "Conversion error in table '{}' for id '{}'",
                table, id
            )),
            DbError::Unknown(msg) => TimeTrackError::Unknown(msg),
        }
    }
}

impl From<ProjectError> for TimeTrackError {
    fn from(error: ProjectError) -> Self {
        match error {
            ProjectError::NotFound => TimeTrackError::ProjectNotFound,
            err => TimeTrackError::Unknown(err.to_string()),
        }
    }
}

pub struct TimeTrackService {
    repository: Arc<TimeTrackRepository>,
    project_service: Arc<ProjectService>,
}

impl TimeTrackService {
    pub fn new(repository: Arc<TimeTrackRepository>, project_service: Arc<ProjectService>) -> Self {
        TimeTrackService {
            repository,
            project_service,
        }
    }

    pub async fn start(
        &self,
        project_id: &str,
        user: &User,
    ) -> Result<(TimeTrack, String), TimeTrackError> {
        let mut project = self.project_service.get(project_id, &user).await?;

        if project.status != ProjectStatus::INACTIVE {
            return Err(TimeTrackError::AlreadyTrackingTime(
                project.name.to_string(),
            ));
        }

        project.status = ProjectStatus::ACTIVE;
        project.modified_at = Some(Utc::now());
        project.modified_by = Some(user.name.to_string());

        // Update the project
        self.project_service.update(&mut project, user).await?;

        let time_track = TimeTrack::new(project_id);
        self.repository.create(&time_track).await?;

        Ok((time_track, project.name))
    }

    pub async fn stop(
        &self,
        project_id: &str,
        user: &User,
    ) -> Result<(TimeTrack, String), TimeTrackError> {
        let mut project = self.project_service.get(project_id, user).await?;

        if project.status != ProjectStatus::ACTIVE {
            return Err(TimeTrackError::NoInProgressTimeTracking(
                project.name.to_string(),
            ));
        }

        // Get the IN_PROGRESS time_track for the project
        let mut time_track = self.repository.get_in_progress(project_id).await?;

        // Update time track item to be finished
        time_track.stopped_at = Some(Utc::now());
        time_track.status = TimeTrackStatus::FINISHED;
        self.repository.update(&time_track).await?;

        // Update the project to be INACTIVE and add duration to total_in_seconds
        project.status = ProjectStatus::INACTIVE;
        if let Some(stopped_at) = time_track.stopped_at {
            let duration = stopped_at - time_track.started_at;
            project.total_in_seconds += duration.num_seconds();
        }

        self.project_service.update(&mut project, user).await?;

        Ok((time_track, project.name))
    }
}
