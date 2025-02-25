use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

use super::user_model::User;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum TimeTrackStatus {
    #[serde(rename = "IN_PROGRESS")]
    InProgress,
    #[serde(rename = "FINISHED")]
    Finished,
}

impl fmt::Display for TimeTrackStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TimeTrackStatus::InProgress => "IN_PROGRESS",
                TimeTrackStatus::Finished => "FINISHED",
            }
        )
    }
}
#[derive(thiserror::Error, Debug)]
pub enum ParseTimeTrackingStatusError {
    #[error("Invalid time tracking status")]
    InvalidStatus,
}

impl FromStr for TimeTrackStatus {
    type Err = ParseTimeTrackingStatusError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "IN_PROGRESS" => Ok(TimeTrackStatus::InProgress),
            "FINISHED" => Ok(TimeTrackStatus::Finished),
            _ => Err(ParseTimeTrackingStatusError::InvalidStatus),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TimeTrack {
    pub id: String, // Changed from String to Uuid for more efficient handling.
    pub project_id: String,
    pub status: TimeTrackStatus,
    pub comment: Option<String>,
    pub started_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub total_duration: Duration,
    pub created_by: String,
}

impl TimeTrack {
    pub fn new<S: Into<String>>(project_id: S, user: &User, comment: Option<String>) -> Self {
        TimeTrack {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.into(),
            status: TimeTrackStatus::InProgress,
            comment: comment,
            started_at: Utc::now(),
            stopped_at: None,
            total_duration: Duration::new(0, 0),
            created_by: user.id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::user_model::UserRole;

    use super::*;

    #[test]
    fn test_time_track_status_display() {
        assert_eq!(
            TimeTrackStatus::InProgress.to_string(),
            "IN_PROGRESS",
            "Expected 'IN_PROGRESS', got '{}'",
            TimeTrackStatus::InProgress
        );
        assert_eq!(
            TimeTrackStatus::Finished.to_string(),
            "FINISHED",
            "Expected 'FINISHED', got '{}'",
            TimeTrackStatus::Finished
        );
    }

    #[test]
    fn test_time_track_status_from_str() {
        assert_eq!(
            "IN_PROGRESS".parse::<TimeTrackStatus>().unwrap(),
            TimeTrackStatus::InProgress,
            "Expected 'TimeTrackStatus::InProgress', got '{:?}'",
            "IN_PROGRESS".parse::<TimeTrackStatus>().unwrap()
        );
        assert_eq!(
            "FINISHED".parse::<TimeTrackStatus>().unwrap(),
            TimeTrackStatus::Finished,
            "Expected 'TimeTrackStatus::Finished', got '{:?}'",
            "FINISHED".parse::<TimeTrackStatus>().unwrap()
        );

        assert!(
            "INVALID".parse::<TimeTrackStatus>().is_err(),
            "Expected 'Err', got '{:?}'",
            "INVALID".parse::<TimeTrackStatus>()
        );
    }

    #[test]
    fn test_time_track_new() {
        let user = User::new("test", &UserRole::User, &Uuid::new_v4().to_string());
        let project_id = "proj_12345";
        let time_track = TimeTrack::new(project_id, &user, None);

        assert_eq!(
            time_track.project_id,
            project_id.to_string(),
            "Expected project_id '{}', got '{}'",
            project_id,
            time_track.project_id
        );
        assert_eq!(
            time_track.status,
            TimeTrackStatus::InProgress,
            "Expected status 'TimeTrackStatus::InProgress', got '{:?}'",
            time_track.status
        );
        assert_eq!(
            time_track.created_by, user.id,
            "Expected created_by '{}', got '{}'",
            user.id, time_track.created_by
        );
        assert!(
            time_track.stopped_at.is_none(),
            "Expected stopped_at 'None', got '{:?}'",
            time_track.stopped_at
        );
        assert_eq!(
            time_track.total_duration,
            Duration::new(0, 0),
            "Expected total_duration '0', got '{:?}'",
            time_track.total_duration
        );
    }
}
