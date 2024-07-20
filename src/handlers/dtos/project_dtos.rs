use crate::models::project_model::{Project, ProjectStatus};

use chrono::{DateTime, Utc};
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
pub struct NewProjectDto {
    #[validate(length(
        min = 1,
        max = 25,
        message = "must be between 1 and 25 characters long"
    ))]
    pub name: String,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for NewProjectDto {
    type Error = Error;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        let limit = 256.bytes();
        let string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => {
                return Outcome::Error((
                    Status::PayloadTooLarge,
                    Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Payload too large",
                    )),
                ))
            }
            Err(e) => return Outcome::Error((Status::InternalServerError, Error::Io(e))),
        };

        let new_project_dto: NewProjectDto = match serde_json::from_str(&string) {
            Ok(dto) => dto,
            Err(e) => return Outcome::Error((Status::UnprocessableEntity, Error::Json(e))),
        };

        if let Err(validation_err) = new_project_dto.validate() {
            return Outcome::Error((
                Status::UnprocessableEntity,
                Error::ValidationError(validation_err),
            ));
        }

        Outcome::Success(new_project_dto)
    }
}
#[derive(Debug, Serialize)]
pub struct ProjectDto {
    pub project_id: String,
    pub name: String,
    pub status: ProjectStatus,
    pub total_in_seconds: i64,
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
            total_in_seconds: p.total_in_seconds,
            created_at: p.created_at,
            modified_at: p.modified_at,
        }
    }
}
