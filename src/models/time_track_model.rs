use chrono::{DateTime, Utc};
use core::fmt;
use serde::Serialize;
use std::str::FromStr;
use uuid::Uuid;

use crate::User;

#[derive(Debug, Serialize)]
pub enum TimeTrackStatus {
    #[serde(rename(serialize = "IN_PROGRESS"))]
    InProgress,
    #[serde(rename(serialize = "FINISHED"))]
    Finished,
}

impl fmt::Display for TimeTrackStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TimeTrackStatus::InProgress => write!(f, "IN_PROGRESS"),
            TimeTrackStatus::Finished => write!(f, "FINISHED"),
        }
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
    pub id: String,
    pub project_id: String,
    pub status: TimeTrackStatus,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stopped_at: Option<DateTime<Utc>>,
    pub created_by: String,
}

impl TimeTrack {
    pub fn new(project_id: &str, user: &User) -> Self {
        TimeTrack {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            status: TimeTrackStatus::InProgress,
            started_at: Utc::now(),
            stopped_at: None,
            created_by: user.name.to_string(),
        }
    }
}
