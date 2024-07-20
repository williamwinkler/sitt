use std::sync::Arc;

use super::{
    dtos::{common_dtos::ErrorResponse, time_track_dtos::TimeTrackDto},
    validation::valid_uuid::ValidateUuid,
};
use crate::{
    services::time_track_service::{TimeTrackError, TimeTrackService},
    User,
};
use rocket::{http::Status, post, response::status, routes, serde::json::Json, Route, State};

pub fn routes() -> Vec<Route> {
    routes![start, stop]
}

#[post("/timetrack/<project_id>/start")]
pub async fn start(
    time_track_service: &State<Arc<TimeTrackService>>,
    user: &State<User>,
    project_id: ValidateUuid,
) -> Result<Json<TimeTrackDto>, status::Custom<Json<ErrorResponse>>> {
    let project_id = project_id.0.to_string();

    match time_track_service.start(&project_id, user).await {
        Ok(res) => Ok(Json(TimeTrackDto::from_time_track_with_project_name(
            res.0, &res.1,
        ))),
        Err(err) => match err {
            TimeTrackError::NotFound => Err(status::Custom(
                Status::NotFound,
                Json(ErrorResponse {
                    error_mesage: err.to_string(),
                }),
            )),
            TimeTrackError::AlreadyTrackingTime(_) => Err(status::Custom(
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

#[post("/timetrack/<project_id>/stop")]
pub async fn stop(
    time_track_service: &State<Arc<TimeTrackService>>,
    user: &State<User>,
    project_id: ValidateUuid,
) -> Result<Json<TimeTrackDto>, status::Custom<Json<ErrorResponse>>> {
    let project_id = project_id.0.to_string();

    match time_track_service.stop(&project_id, user).await {
        Ok(res) => Ok(Json(TimeTrackDto::from_time_track_with_project_name(
            res.0, &res.1,
        ))),
        Err(err) => match err {
            TimeTrackError::NotFound => Err(status::Custom(
                Status::NotFound,
                Json(ErrorResponse {
                    error_mesage: err.to_string(),
                }),
            )),
            TimeTrackError::NoInProgressTimeTracking(_) => Err(status::Custom(
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
