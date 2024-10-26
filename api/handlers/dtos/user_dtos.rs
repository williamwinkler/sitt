use chrono::{DateTime, Utc};
use rocket::{
    data::{self, FromData, ToByteUnit},
    http::Status,
    outcome::Outcome,
    Data, Request,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::user_model::{User, UserRole};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(length(
        min = 1,
        max = 25,
        message = "must be between 1 and 25 characters long"
    ))]
    pub name: String,
    pub role: UserRole,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for CreateUserDto {
    type Error = ();

    async fn from_data(_req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        let limit = 256.bytes();
        let string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Outcome::Error((Status::PayloadTooLarge, ())),
            Err(_) => return Outcome::Error((Status::InternalServerError, ())),
        };

        let create_project_dto: CreateUserDto = match serde_json::from_str(&string) {
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
pub struct UserDto {
    pub id: String,
    pub name: String,
    pub role: UserRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl From<User> for UserDto {
    fn from(u: User) -> Self {
        UserDto {
            id: u.id,
            name: u.name,
            role: u.role,
            api_key: u.api_key,
            created_at: u.created_at,
            created_by: u.created_by,
        }
    }
}
