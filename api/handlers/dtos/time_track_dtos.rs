use crate::models::time_track_model::{TimeTrack, TimeTrackStatus};
use chrono::{DateTime, Utc};
use humantime::format_duration;
use rocket::{
    data::{self, FromData, ToByteUnit},
    http::Status,
    outcome::Outcome,
    Data, Request,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateTimeTrackDto {
    pub project_id: String,
    pub started_at: DateTime<Utc>,
    pub stopped_at: DateTime<Utc>,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for CreateTimeTrackDto {
    type Error = ();

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        let limit = 256.bytes();
        let string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Outcome::Error((Status::PayloadTooLarge, ())),
            Err(_) => return Outcome::Error((Status::InternalServerError, ())),
        };

        let update_time_track_dto: CreateTimeTrackDto = match serde_json::from_str(&string) {
            Ok(value) => value,
            Err(_) => return Outcome::Error((Status::UnprocessableEntity, ())),
        };

        if Uuid::parse_str(&update_time_track_dto.project_id).is_err() {
            return Outcome::Error((Status::UnprocessableEntity, ()));
        }

        Outcome::Success(update_time_track_dto)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeTrackDto {
    pub time_track_id: String,
    pub project_id: String,
    pub project_name: String,
    pub status: TimeTrackStatus,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stopped_at: Option<DateTime<Utc>>,
    pub total_duration: String,
}

impl TimeTrackDto {
    pub fn from_time_track_with_project_name(t: TimeTrack, project_name: String) -> Self {
        TimeTrackDto {
            time_track_id: t.id,
            project_id: t.project_id,
            project_name,
            status: t.status,
            started_at: t.started_at,
            stopped_at: t.stopped_at,
            total_duration: format_duration(t.total_duration).to_string(),
        }
    }
}
