use std::time::Duration;

use crate::models::time_track_model::{TimeTrack, TimeTrackStatus};
use chrono::{DateTime, Utc};
use humantime::{format_duration, parse_duration};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TimeTrackDto {
    pub time_track_id: String,
    pub project_id: String,
    pub project_name: String,
    pub status: TimeTrackStatus,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stopped_at: Option<DateTime<Utc>>,
    pub total_duration: String,
    pub created_by: String,
}

impl TimeTrackDto {
    pub fn from_time_track_with_project_name(t: TimeTrack, project_name: String) -> Self {
        TimeTrackDto {
            time_track_id: t.id,
            project_id: t.project_id,
            project_name: project_name,
            status: t.status,
            started_at: t.started_at,
            stopped_at: t.stopped_at,
            total_duration: format_duration(t.total_duration).to_string(),
            created_by: t.created_by,
        }
    }
}
