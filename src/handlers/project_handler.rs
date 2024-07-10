use super::dtos::common_dtos::ErrorResponse;
use super::validation::valid_uuid::ValidateUuid;
use crate::handlers::dtos::project_dtos::{NewProjectDto, ProjectDto};
use crate::services::project_service::{ProjectError, ProjectService};
use crate::User;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::{delete, get, http::Status, post, response::status, routes, State};

pub fn routes() -> Vec<Route> {
    routes![create, get, get_all, delete]
}

#[post("/projects", format = "application/json", data = "<new_project>")]
pub async fn create(
    project_service: &State<ProjectService>,
    user: &State<User>,
    new_project: NewProjectDto,
) -> Result<status::Created<Json<ProjectDto>>, status::Custom<Json<ErrorResponse>>> {
    let project_name = &new_project.name;

    match project_service.create(project_name, &user).await {
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
    match project_service.get_all(&user).await {
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
    project_id: ValidateUuid,
) -> Result<Json<ProjectDto>, status::Custom<Json<ErrorResponse>>> {
    let project_id = project_id.0.to_string();

    match project_service.get(&project_id, &user).await {
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
    project_id: ValidateUuid,
) -> Result<status::NoContent, status::Custom<Json<ErrorResponse>>> {
    let project_id = project_id.0.to_string();

    match project_service.delete(&project_id, &user).await {
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
