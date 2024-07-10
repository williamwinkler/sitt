extern crate dotenv;
use dotenv::dotenv;
use infrastructure::{
    project_repository::ProjectRepository, time_track_repository::TimeTrackRepository,
};
use std::sync::Arc;

mod handlers;
mod infrastructure;
mod models;
mod services;

struct User {
    name: String,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv().ok();

    // There is only one user for now
    let user = User {
        name: String::from("admin"),
    };

    // Infrastructure
    let database = Arc::new(infrastructure::database::Database::new().await);
    let project_repository = Arc::new(ProjectRepository::new(database.clone()).await);
    let time_track_repository = Arc::new(TimeTrackRepository::new(database.clone()).await);

    // Services
    let project_service =
        Arc::new(services::project_service::ProjectService::new(project_repository.clone()));
    let time_track_service = services::time_track_service::TimeTrackService::new(
        time_track_repository.clone(),
        project_service.clone(),
    );

    let _rocket = rocket::build()
        .manage(user)
        .manage(project_service)
        .manage(time_track_service)
        .mount("/api/v1", handlers::project_handler::routes())
        .mount("/api/v1", handlers::time_track_handler::routes())
        .ignite()
        .await?
        .launch()
        .await?;

    Ok(())
}

// let time_track_repository = Arc::new(TimeTrackRepository::new(database.clone()).await);

// let time_track_service = services::time_track_service::TimeTrackService::new(
//     time_track_repository.clone(),
//     project_service.clone(),
// );
