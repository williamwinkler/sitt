use std::sync::Arc;

use crate::models::user_model::{User, UserRole};
use crate::services::user_service::UserService;
use rocket::{
    outcome::Outcome,
    request::{self, FromRequest},
    Request, State,
};

#[derive(thiserror::Error, Debug)]
pub enum UserValidationError {
    #[error("Missing API key")]
    Missing,
    #[error("Invalid API key")]
    Invalid,
    #[error("Unauthorized request")]
    Unauthorized,
    #[error("Lacks the permissions to perform request")]
    Forbidden,
    #[error("Setup failed")]
    SetupFailed,
}
pub struct UserValidation(pub User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserValidation {
    type Error = UserValidationError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // Retrieve the user service from the Rocket state
        let user_service = match request.guard::<&State<Arc<UserService>>>().await {
            Outcome::Success(service) => service,
            Outcome::Error(_) => {
                return Outcome::Error((
                    rocket::http::Status::InternalServerError,
                    UserValidationError::SetupFailed,
                ))
            }
            Outcome::Forward(_) => {
                return Outcome::Error((
                    rocket::http::Status::Unauthorized,
                    UserValidationError::Unauthorized,
                ))
            }
        };

        // Extract the API key from headers
        let keys: Vec<_> = request.headers().get("x-api-key").collect();
        match keys.len() {
            0 => Outcome::Error((
                rocket::http::Status::Unauthorized,
                UserValidationError::Missing,
            )),
            1 => {
                let api_key = keys[0];
                if api_key.len() != 32 {
                    return Outcome::Error((
                        rocket::http::Status::Unauthorized,
                        UserValidationError::Invalid,
                    ));
                }
                match user_service.get_by_api_key(api_key).await {
                    Ok(user) => Outcome::Success(UserValidation(user)),
                    Err(err) => {
                        eprintln!("{:#?}", err);
                        Outcome::Error((
                            rocket::http::Status::Unauthorized,
                            UserValidationError::Unauthorized,
                        ))
                    }
                }
            }
            _ => Outcome::Error((
                rocket::http::Status::Unauthorized,
                UserValidationError::Invalid,
            )),
        }
    }
}

pub struct AdminValidation(pub User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminValidation {
    type Error = UserValidationError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // Retrieve the user service from the Rocket state
        let user_service = match request.guard::<&State<Arc<UserService>>>().await {
            Outcome::Success(service) => service,
            Outcome::Error(_) => {
                return Outcome::Error((
                    rocket::http::Status::InternalServerError,
                    UserValidationError::SetupFailed,
                ))
            }
            Outcome::Forward(_) => {
                return Outcome::Error((
                    rocket::http::Status::Unauthorized,
                    UserValidationError::Unauthorized,
                ));
            }
        };

        // Extract the API key from headers
        let keys: Vec<_> = request.headers().get("x-api-key").collect();
        match keys.len() {
            0 => Outcome::Error((
                rocket::http::Status::Unauthorized,
                UserValidationError::Missing,
            )),
            1 => {
                let api_key = keys[0];
                if api_key.len() != 32 {
                    return Outcome::Error((
                        rocket::http::Status::Unauthorized,
                        UserValidationError::Invalid,
                    ));
                }
                match user_service.get_by_api_key(api_key).await {
                    Ok(user) => {
                        // Make sure user has role admin
                        if user.role != UserRole::Admin {
                            return Outcome::Error((
                                rocket::http::Status::Forbidden,
                                UserValidationError::Forbidden,
                            ));
                        }

                        // Return admin user
                        Outcome::Success(AdminValidation(user))
                    }
                    Err(err) => {
                        eprintln!("{:#?}", err);
                        Outcome::Error((
                            rocket::http::Status::Unauthorized,
                            UserValidationError::Unauthorized,
                        ))
                    }
                }
            }
            _ => Outcome::Error((
                rocket::http::Status::BadRequest,
                UserValidationError::Invalid,
            )),
        }
    }
}
