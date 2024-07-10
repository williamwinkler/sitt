use super::project_service::{ProjectError, ProjectService};
use crate::{
    infrastructure::{time_track_repository::TimeTrackRepository, DbError},
    models::{
        project_model::ProjectStatus,
        time_track_model::{TimeTrack, TimeTrackStatus},
    },
    User,
};
use chrono::Utc;
use std::sync::Arc;

pub enum TimeTrackError {
    NotFound,
    ProjectNotFound,
    NoInProgressTimeTracking,
    AlreadyTrackingTime,
    UnknownError,
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
        let mut project = match self.project_service.get(project_id, &user).await {
            Ok(project) => project,
            Err(e) => match e {
                ProjectError::NotFound => return Err(TimeTrackError::ProjectNotFound),
                _ => return Err(TimeTrackError::UnknownError),
            },
        };

        if project.status != ProjectStatus::INACTIVE {
            return Err(TimeTrackError::AlreadyTrackingTime);
        }

        project.status = ProjectStatus::ACTIVE;
        project.modified_at = Some(Utc::now());
        project.modified_by = Some(user.name.to_string());

        match self.project_service.update(&mut project, user).await {
            Ok(project) => project,
            Err(_) => return Err(TimeTrackError::UnknownError),
        };

        let time_track = TimeTrack::new(project_id);

        match self.repository.insert(time_track).await {
            Ok(time_track) => Ok((time_track, project.name)),
            Err(_) => Err(TimeTrackError::UnknownError),
        }
    }

    pub async fn stop(&self, project_id: &str, user: &User) -> Result<(TimeTrack, String), TimeTrackError> {
        let mut project = match self.project_service.get(project_id, user).await {
            Ok(project) => project,
            Err(e) => match e {
                ProjectError::NotFound => return Err(TimeTrackError::ProjectNotFound),
                _ => return Err(TimeTrackError::UnknownError),
            },
        };

        if project.status != ProjectStatus::ACTIVE {
            return Err(TimeTrackError::NoInProgressTimeTracking);
        }

        // Get the IN_PROGRESS time_track for the project
        let mut time_track = match self.repository.get_in_progress(project_id).await {
            Ok(time_track) => time_track,
            Err(e) => match e {
                DbError::NotFound => return Err(TimeTrackError::NoInProgressTimeTracking),
                DbError::FailedConvertion(_) => return Err(TimeTrackError::UnknownError),
                DbError::UnknownError => return Err(TimeTrackError::UnknownError),
            },
        };

        // Update time track item to be finished
        time_track.stopped_at = Some(Utc::now());
        time_track.status = TimeTrackStatus::FINISHED;
        time_track = match self.repository.update(time_track).await {
            Ok(time_track) => time_track,
            Err(_) => return Err(TimeTrackError::UnknownError),
        };

        // Update the project to be INACTIVE and add duration to total_in_seconds
        project.status = ProjectStatus::INACTIVE;
        project.modified_at = Some(Utc::now());
        project.modified_by = Some(user.name.to_string());
        if let Some(stopped_at) = time_track.stopped_at {
            let duration = stopped_at - time_track.started_at;
            project.total_in_seconds += duration.num_seconds();
        }
        match self.project_service.update(&mut project, user).await {
            Ok(_) => Ok((time_track, project.name)),
            Err(_) => Err(TimeTrackError::UnknownError),
        }
    }
}
