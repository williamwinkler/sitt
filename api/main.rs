extern crate dotenv;
use dotenv::dotenv;
use infrastructure::{
    project_repository::ProjectRepository, time_track_repository::TimeTrackRepository,
    user_repository::UserRepository,
};
use lambda_web::{is_running_on_lambda, launch_rocket_on_lambda, LambdaError};
use std::sync::Arc;

mod handlers;
mod infrastructure;
mod models;
mod services;

#[rocket::main]
async fn main() -> Result<(), LambdaError> {
    dotenv().ok();

    // Infrastructure
    let database = Arc::new(infrastructure::database::Database::new().await);
    let user_repository = Arc::new(UserRepository::build(database.clone()).await?);
    let project_repository = Arc::new(ProjectRepository::build(database.clone()).await?);
    let time_track_repository = Arc::new(TimeTrackRepository::build(database.clone()).await?);

    // Services
    let project_service = Arc::new(services::project_service::ProjectService::new(
        project_repository.clone(),
        None,
    ));
    let user_service = Arc::new(services::user_service::UserService::new(
        user_repository.clone(),
        project_service.clone(),
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
        .manage(user_service)
        .manage(project_service)
        .manage(time_track_service)
        .mount("/api/v1", handlers::user_handler::routes())
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
