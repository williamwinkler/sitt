use chrono::{DateTime, Utc};

use super::project_service::{ProjectError, ProjectService};
use crate::{
    infrastructure::{database::DbError, time_track_repository::TimeTrackRepository},
    models::{
        project_model::ProjectStatus,
        time_track_model::{TimeTrack, TimeTrackStatus},
        user_model::User,
    },
};
use std::{sync::Arc, time::Duration};

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
        comment: Option<String>,
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

        let time_track = TimeTrack::new(project_id, user, comment);
        self.repository.create(&time_track).await?;

        Ok((time_track, project.name))
    }

    pub async fn stop(
        &self,
        user: &User,
        project_id: &str,
    ) -> Result<(TimeTrack, String), TimeTrackError> {
        let mut project = self.project_service.get(user, project_id).await?;

        if project.status != ProjectStatus::Active {
            return Err(TimeTrackError::NoInProgressTimeTracking(
                project.name.to_string(),
            ));
        }

        // Get the IN_PROGRESS time_track for the project
        let result = self.repository.get_in_progress(user, project_id).await;
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
        let stopped_at = Utc::now();
        time_track.stopped_at = Some(stopped_at);
        time_track.status = TimeTrackStatus::Finished;
        let duration = {
            let time_delta = stopped_at - time_track.started_at;
            Duration::new(time_delta.num_seconds() as u64, 0)
        };
        time_track.total_duration = duration;
        self.repository.update(&time_track).await?;

        // Update the project to be INACTIVE
        // No need to set update_total_duration, because it does that in get
        project.status = ProjectStatus::Inactive;

        self.project_service.update(user, &mut project).await?;

        Ok((time_track, project.name))
    }

    pub async fn create(
        &self,
        user: &User,
        project_id: String,
        started_at: DateTime<Utc>,
        stopped_at: DateTime<Utc>,
        comment: Option<String>
    ) -> Result<(TimeTrack, String), TimeTrackError> {
        let mut project = self.project_service.get(user, &project_id).await?;

        let mut time_track = TimeTrack::new(&project_id, user, comment);
        time_track.started_at = started_at;
        time_track.stopped_at = Some(stopped_at);
        time_track.status = TimeTrackStatus::Finished;
        time_track.total_duration = {
            // Calculate the duration
            let time_delta = stopped_at - started_at;
            Duration::new(time_delta.num_seconds() as u64, 0)
        };
        self.repository.create(&time_track).await?;

        project.total_duration += time_track.total_duration;
        self.project_service.update(user, &mut project).await?;

        Ok((time_track, project.name))
    }

    pub async fn get_all(
        &self,
        user: &User,
        project_id: &str,
    ) -> Result<(Vec<TimeTrack>, String), TimeTrackError> {
        let project = self.project_service.get(user, project_id).await?;
        let mut time_track_items = self.repository.get_all(project_id, user).await?;

        if !time_track_items.is_empty() {
            // Sort the items by started_at in descending order (newest first)
            time_track_items.sort_by(|a, b| a.started_at.cmp(&b.started_at));
        }

        let active_time_track = time_track_items.iter_mut().find(|t| t.status == TimeTrackStatus::InProgress);
        if let Some(time_track) = active_time_track {
            time_track.total_duration = {
                let time_delta = Utc::now() - time_track.started_at;
                Duration::new(time_delta.num_seconds() as u64, 0 )
            }
        }

        Ok((time_track_items, project.name))
    }

    pub async fn get_in_progress(
        &self,
        user: &User,
        project_id: &str,
        project_name: &str,
    ) -> Result<TimeTrack, TimeTrackError> {
        // Get the IN_PROGRESS time_track for the project
        let result = self.repository.get_in_progress(user, project_id).await;
        let mut time_track = match result {
            Ok(time_track) => time_track,
            Err(err) => match err {
                DbError::NotFound => {
                    return Err(TimeTrackError::NoInProgressTimeTracking(
                        project_name.to_string(),
                    ))
                }
                _ => return Err(TimeTrackError::Unknown(format!("{:#?}", err))),
            },
        };

        // Calculate the time it has been running
        time_track.total_duration = {
            let time_delta = Utc::now() - time_track.started_at;
            Duration::new(time_delta.num_seconds() as u64, 0)
        };

        Ok(time_track)
    }

    pub async fn update(
        &self,
        user: &User,
        project_id: String,
        time_track_id: String,
        new_started_at: DateTime<Utc>,
        new_stopped_at: DateTime<Utc>,
    ) -> Result<(TimeTrack, String), TimeTrackError> {
        let mut project = self.project_service.get(user, &project_id).await?;

        let mut time_track = self
            .repository
            .get(project_id.clone(), time_track_id)
            .await?;

        // Reduce the project durtaion with the time_track duration
        project.total_duration -= time_track.total_duration;

        // Update the time track properties
        time_track.started_at = new_started_at;
        time_track.stopped_at = Some(new_stopped_at);
        time_track.total_duration = {
            // Recalculate the duration
            let time_delta = new_stopped_at - new_started_at;
            Duration::new(time_delta.num_seconds() as u64, 0)
        };
        self.repository.update(&time_track).await?;

        // Add the new time track duration to the project total duration
        project.total_duration += time_track.total_duration;
        self.project_service.update(user, &mut project).await?;

        Ok((time_track, project.name))
    }

    pub async fn delete(
        &self,
        user: &User,
        project_id: String,
        time_track_id: String,
    ) -> Result<(), TimeTrackError> {
        // Get the project
        let mut project = self.project_service.get(user, &project_id).await?;

        // Try to delete the time_track and have it returned
        let time_track = self
            .repository
            .delete(user, project_id, time_track_id)
            .await?;

        // Substract the duration from the delete time_track
        project.total_duration -= time_track.total_duration;

        // Update the project with the reduced duration
        self.project_service.update(user, &mut project).await?;

        Ok(())
    }

    pub async fn delete_for_project(
        &self,
        user: &User,
        project_id: &str,
    ) -> Result<(), TimeTrackError> {
        let time_track_items = self.get_all(user, project_id).await?;

        // If there are no time track items, return OK
        if time_track_items.0.is_empty() {
            return Ok(());
        }

        // delete all time track items for the project
        self.repository.delete_for_project(project_id).await?;

        Ok(())
    }
}
