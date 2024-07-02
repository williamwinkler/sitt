use crate::handlers::dtos::project_dtos::{NewProjectDto, ProjectDto};
use crate::services::project_service::{ProjectError, ProjectService};
use rocket::delete;
use rocket::serde::json::Json;
use rocket::{get, http::Status, post, response::status, State};

use super::dtos::common_dtos::ErrorResponse;

static USER: &str = "admin";

#[post("/projects", format = "application/json", data = "<new_project>")]
pub async fn create(
    project_service: &State<ProjectService>,
    new_project: Json<NewProjectDto>,
) -> Result<status::Created<Json<ProjectDto>>, status::Custom<Json<ErrorResponse>>> {
    let project_name = &new_project.name;
    let created_by = USER;
    match project_service.create(project_name, created_by).await {
        Ok(project) => Ok(status::Created::new("/projects").body(Json(ProjectDto::from(project)))),
        Err(err) => match err {
            ProjectError::ProjectExistsWithSameName => Err(status::Custom(
                Status::BadRequest,
                Json(ErrorResponse {
                    error: format!("A project already exists with name: {project_name}")
                        .to_string(),
                }),
            )),
            _ => Err(ErrorResponse::internal_server_error()),
        },
    }
}

#[get("/projects")]
pub async fn get_all(
    project_service: &State<ProjectService>,
) -> Result<Json<Vec<ProjectDto>>, status::Custom<Json<ErrorResponse>>> {
    match project_service.get_all("admin").await {
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
    project_id: &str,
) -> Result<Json<ProjectDto>, status::Custom<Json<ErrorResponse>>> {
    match project_service.get(project_id, "admin").await {
        Ok(project) => Ok(Json(ProjectDto::from(project))),
        Err(err) => match err {
            ProjectError::NotFound => Err(ErrorResponse::custom(
                Status::NotFound,
                format!("No project found with id: {project_id}").as_str(),
            )),
            _ => Err(ErrorResponse::internal_server_error()),
        },
    }
}

#[delete("/projects/<project_id>")]
pub async fn delete(
    project_service: &State<ProjectService>,
    project_id: &str,
) -> Result<status::NoContent, status::Custom<Json<ErrorResponse>>> {
    match project_service.delete(project_id, "admin").await {
        Ok(_) => Ok(status::NoContent),
        Err(err) => match err {
            ProjectError::NotFound => Err(status::Custom(
                Status::NotFound,
                Json(ErrorResponse {
                    error: format!("No project found with id: {project_id}").to_string(),
                }),
            )),
            _ => Err(status::Custom(
                Status::InternalServerError,
                Json(ErrorResponse {
                    error: "An unknown error occurred".to_string(),
                }),
            )),
        },
    }
}
