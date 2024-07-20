use super::{
    dtos::{common_dtos::ErrorResponse, time_track_dtos::TimeTrackDto},
    validation::valid_uuid::ValidateUuid,
};
use crate::{services::time_track_service::{TimeTrackError, TimeTrackService}, User};
use rocket::{get, http::Status, post, response::status, routes, serde::json::Json, Route, State};
use std::sync::Arc;

pub fn routes() -> Vec<Route> {
    routes![start, stop, get]
}

#[post("/timetrack/<project_id>/start")]
pub async fn start(
    time_track_service: &State<Arc<TimeTrackService>>,
    user: &State<User>,
    project_id: ValidateUuid,
) -> Result<Json<TimeTrackDto>, status::Custom<Json<ErrorResponse>>> {
    let project_id = project_id.0.to_string();

    match time_track_service.start(user, &project_id).await {
        Ok(res) => Ok(Json(TimeTrackDto::from_time_track_with_project_name(
            res.0, res.1,
        ))),
        Err(err) => match err {
            TimeTrackError::ProjectNotFound => Err(status::Custom(
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

    match time_track_service.stop(user, &project_id).await {
        Ok(res) => Ok(Json(TimeTrackDto::from_time_track_with_project_name(
            res.0, res.1,
        ))),
        Err(err) => match err {
            TimeTrackError::ProjectNotFound => Err(status::Custom(
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

#[get("/timetrack/<project_id>")]
pub async fn get(
    time_track_service: &State<Arc<TimeTrackService>>,
    user: &State<User>,
    project_id: ValidateUuid,
) -> Result<Json<Vec<TimeTrackDto>>, status::Custom<Json<ErrorResponse>>> {
    let project_id = project_id.0.to_string();

    match time_track_service.get_all(user, &project_id).await {
        Ok(result) => {
            let time_track_items_dto = result
                .0
                .into_iter()
                .map(|tt| TimeTrackDto::from_time_track_with_project_name(tt, result.1.clone()))
                .collect();
            Ok(Json(time_track_items_dto))
        }
        Err(err) => match err {
            TimeTrackError::ProjectNotFound => Err(status::Custom(
                Status::NotFound,
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
