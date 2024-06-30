use crate::handlers::dtos::project_dtos::{NewProjectDto, ProjectDto};
use crate::services::project_service::ProjectService;
use rocket::delete;
use rocket::{get, http::Status, post, response::status, serde::json::Json, State};

#[post("/projects", format = "application/json", data = "<new_project>")]
pub async fn create(
    project_service: &State<ProjectService>,
    new_project: Json<NewProjectDto>,
) -> Result<status::Created<Json<ProjectDto>>, status::Custom<Json<String>>> {
    let project_name = &new_project.name;

    match project_service.create(project_name).await {
        Ok(project) => Ok(status::Created::new("/projects").body(Json(ProjectDto::from(project)))),
        Err(err) => Err(status::Custom(Status::InternalServerError, Json(err))),
    }
}

#[get("/projects")]
pub async fn get_all(
    project_service: &State<ProjectService>,
) -> Result<Json<Vec<ProjectDto>>, Status> {
    project_service
        .get_all("admin")
        .await
        .map(|projects| {
            let project_dtos: Vec<ProjectDto> =
                projects.into_iter().map(ProjectDto::from).collect();
            Json(project_dtos)
        })
        .map_err(|_| Status::InternalServerError)
}

#[get("/projects/<project_id>")]
pub async fn get(
    project_service: &State<ProjectService>,
    project_id: &str,
) -> Result<Json<ProjectDto>, Status> {
    match project_service.get(project_id, "admin").await {
        Ok(project) => Ok(Json(ProjectDto::from(project))),
        Err(_) => Err(Status::NotFound),
    }
}

// #[put(
//     "/projects/<project_id>",
//     format = "application/json",
//     data = "<update_project>"
// )]
// pub async fn update_project(
//     project_name: &str,
//     update_project: Json<UpdateProjectDto>,
// ) -> Result<Json<ProjectDto>, status::Custom<Json<String>>> {
//     match project_service::update_project(project_name, update_project.into_inner()).await {
//         Ok(project) => Ok(Json(project)),
//         Err(err) => Err(status::Custom(Status::InternalServerError, Json(err))),
//     }
// }

#[delete("/projects/<project_id>")]
pub async fn delete(project_service: &State<ProjectService>, project_id: &str) -> Status {
    match project_service.delete(project_id, "admin").await {
        Ok(_) => Status::NoContent,
        Err(msg) => {
            println!("{:#?}", msg);
            Status::InternalServerError
        }
    }
}
