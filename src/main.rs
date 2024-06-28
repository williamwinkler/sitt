use rocket::Rocket;
use std::sync::Arc;

mod handlers;
mod infrastructure;
mod models;
mod routes;
mod services;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let database = Arc::new(infrastructure::database::Database::new().await);
    let project_respository =
        Arc::new(infrastructure::project_repository::ProjectRepository::new(database).await);

    let project_service = services::project_service::ProjectService::new(project_respository);

    let _rocket = rocket::build()
        .manage(project_service)
        .mount("/api/v1", routes::project_routes::routes())
        .ignite()
        .await?
        .launch()
        .await?;

    Ok(())
}
