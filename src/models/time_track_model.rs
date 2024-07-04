use core::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum TimeTrackStatus {
    IN_PROGRESS,
    FINISHED,
}

impl fmt::Display for TimeTrackStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TimeTrackStatus::IN_PROGRESS => write!(f, "IN_PROGRESS"),
            TimeTrackStatus::FINISHED => write!(f, "FINISHED"),
        }
    }
}

impl TimeTrackStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "IN_PROGRESS" => Some(TimeTrackStatus::IN_PROGRESS),
            "FINISHED" => Some(TimeTrackStatus::FINISHED),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeTrack {
    pub id: String,
    pub project_id: String,
    pub status: TimeTrackStatus,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<DateTime<Utc>>,
}

impl TimeTrack {
    pub fn new(project_id: &str) -> Self {
        TimeTrack {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            status: TimeTrackStatus::IN_PROGRESS,
            started_at: Utc::now(),
            finished_at: None,
        }
    }
}
