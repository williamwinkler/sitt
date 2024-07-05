use std::{ops::Add, sync::Arc};

use chrono::{Timelike, Utc};

use crate::{
    infrastructure::{time_track_repository::TimeTrackRepository, DbErrors},
    models::{
        project_model::ProjectStatus,
        time_track_model::{TimeTrack, TimeTrackStatus},
    },
};

use super::project_service::{ProjectError, ProjectService};

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

    pub async fn start_for_project_id(
        &self,
        project_id: &str,
        created_by: &str,
    ) -> Result<TimeTrack, TimeTrackError> {
        let project = match self.project_service.get(project_id, created_by).await {
            Ok(project) => project,
            Err(e) => match e {
                ProjectError::NotFound => return Err(TimeTrackError::ProjectNotFound),
                _ => return Err(TimeTrackError::UnknownError),
            },
        };

        if project.status != ProjectStatus::INACTIVE {
            return Err(TimeTrackError::AlreadyTrackingTime);
        }

        let time_track = TimeTrack::new(project_id);

        match self.repository.insert(time_track).await {
            Ok(time_track) => Ok(time_track),
            Err(_) => Err(TimeTrackError::UnknownError),
        }
    }

    pub async fn stop_for_project_id(
        &self,
        project_id: &str,
        created_by: &str,
    ) -> Result<TimeTrack, TimeTrackError> {
        let mut project = match self.project_service.get(project_id, created_by).await {
            Ok(project) => project,
            Err(e) => match e {
                ProjectError::NotFound => return Err(TimeTrackError::ProjectNotFound),
                _ => return Err(TimeTrackError::UnknownError),
            },
        };

        if project.status != ProjectStatus::ACTIVE {
            return Err(TimeTrackError::NoInProgressTimeTracking);
        }

        let mut time_track = match self.repository.get_in_progress(project_id).await {
            Ok(time_track) => time_track,
            Err(e) => match e {
                DbErrors::NotFound => return Err(TimeTrackError::NoInProgressTimeTracking),
                DbErrors::FailedConvertion(_) => return Err(TimeTrackError::UnknownError),
                DbErrors::UnknownError => return Err(TimeTrackError::UnknownError),
            },
        };

        // Update time track item to be finished
        time_track.stopped_at = Some(Utc::now());
        time_track.status = TimeTrackStatus::FINISHED;

        time_track = match self.repository.update(time_track).await {
            Ok(time_track) => time_track,
            Err(_) => return Err(TimeTrackError::UnknownError),
        };

        // Update the project to be INACTIVE and to total_in_seconds
        project.status = ProjectStatus::INACTIVE;
        project.modified_at = Some(Utc::now());
        project.modified_by = Some(created_by.to_string());
        if let Some(stopped_at) = time_track.stopped_at {
            let duration = stopped_at - time_track.started_at;
            project.total_in_seconds += duration.num_seconds();
        }

        // TODO: UPDATE project

        Ok(time_track)
    }
}
