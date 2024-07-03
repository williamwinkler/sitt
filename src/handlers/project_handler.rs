use crate::handlers::dtos::project_dtos::{NewProjectDto, ProjectDto};
use crate::services::project_service::{ProjectError, ProjectService};
use crate::User;
use rocket::serde::json::Json;
use rocket::{delete, get, http::Status, post, response::status, State};
use validator::Validate;

use super::dtos::common_dtos::ErrorResponse;

#[post("/projects", format = "application/json", data = "<new_project>")]
pub async fn create(
    project_service: &State<ProjectService>,
    user: &State<User>,
    new_project: Json<NewProjectDto>,
) -> Result<status::Created<Json<ProjectDto>>, status::Custom<Json<ErrorResponse>>> {
    let new_project = new_project.into_inner();

    // Validate the DTO
    // TODO: use Rocket Guards instead
    if let Err(validation_err) = new_project.validate() {
        return Err(ErrorResponse::validation_error(validation_err));
    }

    let project_name = &new_project.name;
    let created_by = &user.name;

    match project_service.create(project_name, created_by).await {
        Ok(project) => Ok(status::Created::new("/projects").body(Json(ProjectDto::from(project)))),
        Err(err) => match err {
            ProjectError::ProjectExistsWithSameName => Err(ErrorResponse::custom(
                Status::BadRequest,
                &format!("A project already exists with name: {project_name}"),
                None,
            )),
            _ => Err(ErrorResponse::internal_server_error()),
        },
    }
}

#[get("/projects")]
pub async fn get_all(
    project_service: &State<ProjectService>,
    user: &State<User>,
) -> Result<Json<Vec<ProjectDto>>, status::Custom<Json<ErrorResponse>>> {
    let created_by = &user.name;

    match project_service.get_all(created_by).await {
        Ok(projects) => {
            let project_dtos = projects.into_iter().map(ProjectDto::from).collect();
            Ok(Json(project_dtos))
        }
        Err(err) => match err {
            _ => Err(ErrorResponse::internal_server_error()),
        },
    }
}

#[get("/projects/<project_id>")]
pub async fn get(
    project_service: &State<ProjectService>,
    user: &State<User>,
    project_id: &str,
) -> Result<Json<ProjectDto>, status::Custom<Json<ErrorResponse>>> {
    let created_by = &user.name;

    match project_service.get(project_id, created_by).await {
        Ok(project) => Ok(Json(ProjectDto::from(project))),
        Err(err) => match err {
            ProjectError::NotFound => Err(ErrorResponse::custom(
                Status::NotFound,
                &format!("No project found with id: {project_id}"),
                None,
            )),
            _ => Err(ErrorResponse::internal_server_error()),
        },
    }
}

#[delete("/projects/<project_id>")]
pub async fn delete(
    project_service: &State<ProjectService>,
    user: &State<User>,
    project_id: &str,
) -> Result<status::NoContent, status::Custom<Json<ErrorResponse>>> {
    let created_by = &user.name;

    match project_service.delete(project_id, created_by).await {
        Ok(_) => Ok(status::NoContent),
        Err(err) => match err {
            ProjectError::NotFound => Err(ErrorResponse::custom(
                Status::NotFound,
                &format!("No project found with id: {project_id}"),
                None,
            )),
            _ => Err(ErrorResponse::custom(
                Status::InternalServerError,
                "An unknown error occurred",
                None,
            )),
        },
    }
}
