use super::validation::uuid_validation::UuidValidation;
use super::{
    dtos::{
        common_dtos::ErrorResponse,
        user_dtos::{CreateUserDto, UserDto},
    },
    validation::user_validation::AdminValidation,
};
use crate::services::user_service::{UserError, UserService};
use rocket::get;
use rocket::{
    delete, http::Status, post, response::status, routes, serde::json::Json, Route, State,
};
use std::sync::Arc;

pub fn routes() -> Vec<Route> {
    routes![create, get, get_all, delete]
}

#[post("/users", format = "application/json", data = "<create_user_dto>")]
pub async fn create(
    user_service: &State<Arc<UserService>>,
    admin_user: AdminValidation,
    create_user_dto: CreateUserDto,
) -> Result<status::Created<Json<UserDto>>, status::Custom<Json<ErrorResponse>>> {
    let admin_user = &admin_user.0;
    let name = &create_user_dto.name;
    let role = &create_user_dto.role;

    match user_service.create(name, role, admin_user).await {
        Ok(user) => Ok(status::Created::new("/users").body(Json(UserDto::from(user)))),
        Err(err) => {
            eprintln!("{}", err);
            Err(status::Custom(
                Status::InternalServerError,
                Json(ErrorResponse {
                    error_message: String::from("An internal error occurred"),
                }),
            ))
        }
    }
}

#[get("/users/<user_id>?<include_api_key>")]
pub async fn get(
    user_service: &State<Arc<UserService>>,
    admin_user: AdminValidation,
    user_id: UuidValidation,
    include_api_key: Option<bool>,
) -> Result<Json<UserDto>, status::Custom<Json<ErrorResponse>>> {
    let _ = admin_user.0;
    let user_id = &user_id.0.to_string();
    let include_api_key: bool = include_api_key.unwrap_or(false);

    match user_service.get_by_id(user_id, include_api_key).await {
        Ok(user) => Ok(Json(UserDto::from(user))),
        Err(err) => match err {
            UserError::NotFound => Err(status::Custom(
                Status::NotFound,
                Json(ErrorResponse {
                    error_message: err.to_string(),
                }),
            )),
            _ => {
                eprintln!("{}", err);
                Err(status::Custom(
                    Status::InternalServerError,
                    Json(ErrorResponse {
                        error_message: String::from("An internal error occurred"),
                    }),
                ))
            }
        },
    }
}

#[get("/users")]
pub async fn get_all(
    user_service: &State<Arc<UserService>>,
    admin_user: AdminValidation,
) -> Result<Json<Vec<UserDto>>, status::Custom<Json<ErrorResponse>>> {
    let _ = admin_user.0;

    match user_service.get_all().await {
        Ok(users) => {
            let user_dtos: Vec<UserDto> = users.into_iter().map(UserDto::from).collect();
            Ok(Json(user_dtos))
        }
        Err(err) => {
            eprintln!("{}", err);
            Err(status::Custom(
                Status::InternalServerError,
                Json(ErrorResponse {
                    error_message: String::from("An internal error occurred"),
                }),
            ))
        }
    }
}

#[delete("/users/<user_id>")]
pub async fn delete(
    user_service: &State<Arc<UserService>>,
    admin: AdminValidation,
    user_id: UuidValidation,
) -> Result<status::NoContent, status::Custom<Json<ErrorResponse>>> {
    let admin = &admin.0;
    let user_id = &user_id.0.to_string();

    if admin.id == *user_id {
        return Err(status::Custom(
            Status::BadRequest,
            Json(ErrorResponse {
                error_message: String::from("You can't delete yourself"),
            }),
        ));
    }

    match user_service.delete(user_id).await {
        Ok(_) => Ok(status::NoContent),
        Err(err) => match err {
            UserError::NotFound => Err(status::Custom(
                Status::NotFound,
                Json(ErrorResponse {
                    error_message: err.to_string(),
                }),
            )),
            _ => {
                eprintln!("{}", err);
                Err(status::Custom(
                    Status::InternalServerError,
                    Json(ErrorResponse {
                        error_message: String::from("An internal error occurred"),
                    }),
                ))
            }
        },
    }
}
