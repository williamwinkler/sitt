use std::sync::Arc;

use super::dtos::common_dtos::ErrorResponse;
use super::validation::valid_uuid::ValidateUuid;
use crate::handlers::dtos::project_dtos::{CreateProjectDto, ProjectDto};
use crate::services::project_service::{ProjectError, ProjectService};
use crate::User;
use rocket::serde::json::Json;
use rocket::{delete, get, http::Status, post, response::status, routes, State};
use rocket::{put, Route};

pub fn routes() -> Vec<Route> {
    routes![create, get, get_all, update, delete]
}

#[post("/projects", format = "application/json", data = "<new_project>")]
pub async fn create(
    project_service: &State<Arc<ProjectService>>,
    user: &State<User>,
    new_project: CreateProjectDto,
) -> Result<status::Created<Json<ProjectDto>>, status::Custom<Json<ErrorResponse>>> {
    let project_name = new_project.name;

    match project_service.create(&user, project_name).await {
        Ok(project) => Ok(status::Created::new("/projects").body(Json(ProjectDto::from(project)))),
        Err(err) => match err {
            ProjectError::NotFound => Err(status::Custom(
                Status::NotFound,
                Json(ErrorResponse {
                    error_mesage: err.to_string(),
                }),
            )),
            ProjectError::ProjectExistsWithSameName(_) => Err(status::Custom(
                Status::Conflict,
                Json(ErrorResponse {
                    error_mesage: err.to_string(),
                }),
            )),
            ProjectError::TooManyProjects => Err(status::Custom(
                Status::BadRequest,
                Json(ErrorResponse {
                    error_mesage: err.to_string(),
                }),
            )),
            _ => {
                eprintln!("{}", err.to_string());
                Err(status::Custom(
                    Status::InternalServerError,
                    Json(ErrorResponse {
                        error_mesage: String::from("An internal error occurred"),
                    }),
                ))
            }
        },
    }
}

#[get("/projects")]
pub async fn get_all(
    project_service: &State<Arc<ProjectService>>,
    user: &State<User>,
) -> Result<Json<Vec<ProjectDto>>, status::Custom<Json<ErrorResponse>>> {
    match project_service.get_all(&user).await {
        Ok(projects) => {
            let project_dtos = projects.into_iter().map(ProjectDto::from).collect();
            Ok(Json(project_dtos))
        }
        Err(err) => {
            eprintln!("{}", err.to_string());
            Err(status::Custom(
                Status::InternalServerError,
                Json(ErrorResponse {
                    error_mesage: String::from("An internal error occurred"),
                }),
            ))
        }
    }
}

#[get("/projects/<project_id>")]
pub async fn get(
    project_service: &State<Arc<ProjectService>>,
    user: &State<User>,
    project_id: ValidateUuid,
) -> Result<Json<ProjectDto>, status::Custom<Json<ErrorResponse>>> {
    let project_id = project_id.0.to_string();

    match project_service.get(user, &project_id).await {
        Ok(project) => Ok(Json(ProjectDto::from(project))),
        Err(err) => match err {
            ProjectError::NotFound => Err(status::Custom(
                Status::NotFound,
                Json(ErrorResponse {
                    error_mesage: err.to_string(),
                }),
            )),
            _ => {
                eprintln!("{}", err.to_string());
                Err(status::Custom(
                    Status::InternalServerError,
                    Json(ErrorResponse {
                        error_mesage: String::from("An internal error occurred"),
                    }),
                ))
            }
        },
    }
}

#[put(
    "/projects/<project_id>",
    format = "application/json",
    data = "<update_project>"
)]
pub async fn update(
    project_service: &State<Arc<ProjectService>>,
    user: &State<User>,
    project_id: ValidateUuid,
    update_project: CreateProjectDto,
) -> Result<Json<ProjectDto>, status::Custom<Json<ErrorResponse>>> {
    let project_id = project_id.0.to_string();
    let new_project_name = update_project.name;

    match project_service
        .update_name(user, project_id, new_project_name)
        .await
    {
        Ok(project) => Ok(Json(ProjectDto::from(project))),
        Err(err) => match err {
            ProjectError::NotFound => Err(status::Custom(
                Status::NotFound,
                Json(ErrorResponse {
                    error_mesage: err.to_string(),
                }),
            )),
            _ => {
                eprintln!("{}", err.to_string());
                Err(status::Custom(
                    Status::InternalServerError,
                    Json(ErrorResponse {
                        error_mesage: String::from("An internal error occurred"),
                    }),
                ))
            }
        },
    }
}

#[delete("/projects/<project_id>")]
pub async fn delete(
    project_service: &State<Arc<ProjectService>>,
    user: &State<User>,
    project_id: ValidateUuid,
) -> Result<status::NoContent, status::Custom<Json<ErrorResponse>>> {
    let project_id = project_id.0.to_string();

    match project_service.delete(user, &project_id).await {
        Ok(_) => Ok(status::NoContent),
        Err(err) => match err {
            ProjectError::NotFound => Err(status::Custom(
                Status::NotFound,
                Json(ErrorResponse {
                    error_mesage: err.to_string(),
                }),
            )),
            _ => {
                eprintln!("{}", err.to_string());
                Err(status::Custom(
                    Status::InternalServerError,
                    Json(ErrorResponse {
                        error_mesage: String::from("An internal error occurred"),
                    }),
                ))
            }
        },
    }
}
