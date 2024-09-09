use crate::models::project_model::{Project, ProjectStatus};
use chrono::{DateTime, Utc};
use humantime::format_duration;
use rocket::data::{self, Data, FromData, ToByteUnit};
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::Request;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    ValidationError(ValidationErrors),
    Json(serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(crate = "rocket::serde")]
pub struct CreateProjectDto {
    #[validate(length(
        min = 1,
        max = 25,
        message = "must be between 1 and 25 characters long"
    ))]
    pub name: String,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for CreateProjectDto {
    type Error = ();

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        let limit = 256.bytes();
        let string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Outcome::Error((Status::PayloadTooLarge, ())),
            Err(_) => return Outcome::Error((Status::InternalServerError, ())),
        };

        let create_project_dto: CreateProjectDto = match serde_json::from_str(&string) {
            Ok(value) => value,
            Err(_) => return Outcome::Error((Status::UnprocessableEntity, ())),
        };

        if create_project_dto.validate().is_err() {
            return Outcome::Error((Status::UnprocessableEntity, ()));
        }

        Outcome::Success(create_project_dto)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDto {
    pub project_id: String,
    pub name: String,
    pub status: ProjectStatus,
    pub total_duration: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
}

impl From<Project> for ProjectDto {
    fn from(p: Project) -> Self {
        ProjectDto {
            project_id: p.id,
            name: p.name,
            status: p.status,
            total_duration: format_duration(p.total_duration).to_string(),
            created_at: p.created_at,
            modified_at: p.modified_at,
        }
    }
}
