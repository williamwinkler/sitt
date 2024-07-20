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
    #[error("No time tracking is in progress for project '{0}'.")]
    NoInProgressTimeTracking(String),
    #[error("Time tracking is already in progress on project '{0}'")]
    AlreadyTrackingTime(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<DbError> for TimeTrackError {
    fn from(error: DbError) -> Self {
        match error {
            DbError::NotFound => TimeTrackError::NotFound,
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

#[derive(Debug)]
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
        user: &User,
        project_id: &str,
    ) -> Result<(TimeTrack, String), TimeTrackError> {
        let mut project = self.project_service.get(user, project_id).await?;

        if project.status != ProjectStatus::Inactive {
            return Err(TimeTrackError::AlreadyTrackingTime(
                project.name.to_string(),
            ));
        }

        // Update the project
        project.status = ProjectStatus::Active;
        self.project_service.update(user, &mut project).await?;

        let time_track = TimeTrack::new(project_id, user);
        self.repository.create(&time_track).await?;

        Ok((time_track, project.name))
    }

    pub async fn stop(
        &self,
        user: &User,
        project_id: &str,
    ) -> Result<(TimeTrack, String), TimeTrackError> {
        let mut project = self.project_service.get(user, project_id,).await?;

        if project.status != ProjectStatus::Active {
            return Err(TimeTrackError::NoInProgressTimeTracking(
                project.name.to_string(),
            ));
        }

        // Get the IN_PROGRESS time_track for the project
        let result = self.repository.get_in_progress(project_id).await;
        let mut time_track = match result {
            Ok(time_track) => time_track,
            Err(err) => match err {
                DbError::NotFound => {
                    return Err(TimeTrackError::NoInProgressTimeTracking(project.name))
                }
                _ => return Err(TimeTrackError::Unknown(format!("{:#?}", err))),
            },
        };

        // Update time track item to be finished
        time_track.stopped_at = Some(Utc::now());
        time_track.status = TimeTrackStatus::Finished;
        self.repository.update(&time_track).await?;

        // Update the project to be INACTIVE and add duration to total_in_seconds
        project.status = ProjectStatus::Inactive;
        if let Some(stopped_at) = time_track.stopped_at {
            let duration = stopped_at - time_track.started_at;
            project.total_in_seconds += duration.num_seconds();
        }

        self.project_service.update(user, &mut project,).await?;

        Ok((time_track, project.name))
    }

    pub async fn get_all(&self, user: &User, project_id: &str) -> Result<(Vec<TimeTrack>, String), TimeTrackError> {
        let project = self.project_service.get(user, project_id).await?;
        let mut time_track_items = self.repository.get_all(project_id, user).await?;

        // Sort the items by started_at in descending order (newest first)
        time_track_items.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        Ok((time_track_items, project.name))
    }

    pub async fn delete_for_project(&self, user: &User, project_id: &str) -> Result<(), TimeTrackError> {
        let time_track_items = self.get_all(user, project_id).await?;

        // If there are no time track items, return OK
        if time_track_items.0.is_empty() {
            return Ok(())
        }

        let result = self.repository.delete_for_project(project_id).await?;

        Ok(result)
    }
}
