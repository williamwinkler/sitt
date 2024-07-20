use crate::models::time_track_model::{TimeTrack, TimeTrackStatus};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TimeTrackDto {
    pub time_track_id: String,
    pub project_id: String,
    pub project_name: String,
    pub status: TimeTrackStatus,
    pub started_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
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
            created_by: t.created_by,
        }
    }
}
