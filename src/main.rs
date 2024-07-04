use std::sync::Arc;

mod handlers;
mod infrastructure;
mod models;
mod routes;
mod services;

struct User {
    name: String,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // The user will always be 'admin' for now. Later I might add support for more.
    let created_by = User {
        name: String::from("admin"),
    };

    // Infrastructure
    let database = Arc::new(infrastructure::database::Database::new().await);
    let project_respository =
        Arc::new(infrastructure::project_repository::ProjectRepository::new(database).await);

    // Services
    let project_service = services::project_service::ProjectService::new(project_respository);

    let _rocket = rocket::build()
        .manage(created_by)
        .manage(project_service)
        .mount("/api/v1", routes::project_routes::routes())
        .ignite()
        .await?
        .launch()
        .await?;

    Ok(())
}
