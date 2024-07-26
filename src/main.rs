extern crate dotenv;
use dotenv::dotenv;
use infrastructure::{
    project_repository::ProjectRepository, time_track_repository::TimeTrackRepository,
};
use lambda_web::{is_running_on_lambda, launch_rocket_on_lambda, LambdaError};
use std::sync::Arc;

mod handlers;
mod infrastructure;
mod models;
mod services;

struct User {
    name: String,
}

#[rocket::main]
async fn main() -> Result<(), LambdaError> {
    dotenv().ok();

    // There is only one user for now
    let user = User {
        name: String::from("admin"),
    };

    // Infrastructure
    let database = Arc::new(infrastructure::database::Database::new().await);
    let project_repository = Arc::new(ProjectRepository::build(database.clone()).await?);
    let time_track_repository = Arc::new(TimeTrackRepository::build(database.clone()).await?);

    // Services
    let project_service = Arc::new(services::project_service::ProjectService::new(
        project_repository.clone(),
        None,
    ));
    let time_track_service = Arc::new(services::time_track_service::TimeTrackService::new(
        time_track_repository.clone(),
        project_service.clone(),
    ));

    project_service
        .set_time_track_service(time_track_service.clone())
        .await;

    // Setup Rocket
    let rocket = rocket::build()
        .manage(user)
        .manage(project_service)
        .manage(time_track_service)
        .mount("/api/v1", handlers::project_handler::routes())
        .mount("/api/v1", handlers::time_track_handler::routes());

    if is_running_on_lambda() {
        // Launch on AWS Lambda
        launch_rocket_on_lambda(rocket).await?;
    } else {
        // Launch local server
        let _ = rocket.launch().await?;
    }

    Ok(())
}
