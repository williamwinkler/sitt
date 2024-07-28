use super::{
    dtos::{
        common_dtos::ErrorResponse,
        time_track_dtos::{CreateTimeTrackDto, TimeTrackDto},
    },
    validation::{user_validation::UserValidation, uuid_validation::UuidValidation},
};
use crate::{
    models::user_model::User,
    services::time_track_service::{TimeTrackError, TimeTrackService},
};
use rocket::{
    delete, get, http::Status, post, put, response::status, routes, serde::json::Json, Route, State,
};
use std::sync::Arc;

pub fn routes() -> Vec<Route> {
    routes![start, stop, create, get, update, delete]
}

#[post("/timetrack/<project_id>/start")]
pub async fn start(
    time_track_service: &State<Arc<TimeTrackService>>,
    user: UserValidation,
    project_id: UuidValidation,
) -> Result<Json<TimeTrackDto>, status::Custom<Json<ErrorResponse>>> {
    let user = &user.0;
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
    user: UserValidation,
    project_id: UuidValidation,
) -> Result<Json<TimeTrackDto>, status::Custom<Json<ErrorResponse>>> {
    let user = &user.0;
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

#[post(
    "/timetrack",
    format = "application/json",
    data = "<create_time_track_dto>"
)]
pub async fn create(
    time_track_service: &State<Arc<TimeTrackService>>,
    user: UserValidation,
    create_time_track_dto: CreateTimeTrackDto,
) -> Result<Json<TimeTrackDto>, status::Custom<Json<ErrorResponse>>> {
    let user = &user.0;
    let project_id = create_time_track_dto.project_id;
    let started_at = create_time_track_dto.started_at;
    let stopped_at = create_time_track_dto.stopped_at;

    match time_track_service
        .create(user, project_id, started_at, stopped_at)
        .await
    {
        Ok(res) => Ok(Json(TimeTrackDto::from_time_track_with_project_name(
            res.0, res.1,
        ))),
        Err(err) => match err {
            TimeTrackError::NotFound => Err(status::Custom(
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

#[get("/timetrack/<project_id>")]
pub async fn get(
    time_track_service: &State<Arc<TimeTrackService>>,
    user: UserValidation,
    project_id: UuidValidation,
) -> Result<Json<Vec<TimeTrackDto>>, status::Custom<Json<ErrorResponse>>> {
    let user = &user.0;
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

#[put(
    "/timetrack/<time_track_id>",
    format = "application/json",
    data = "<update_time_track_dto>"
)]
pub async fn update(
    time_track_service: &State<Arc<TimeTrackService>>,
    user: UserValidation,
    time_track_id: UuidValidation,
    update_time_track_dto: CreateTimeTrackDto,
) -> Result<Json<TimeTrackDto>, status::Custom<Json<ErrorResponse>>> {
    let user = &user.0;
    let time_track_id = time_track_id.0.to_string();
    let project_id = update_time_track_dto.project_id;
    let new_started_at = update_time_track_dto.started_at;
    let new_stopped_at = update_time_track_dto.stopped_at;

    match time_track_service
        .update(
            user,
            project_id,
            time_track_id,
            new_started_at,
            new_stopped_at,
        )
        .await
    {
        Ok(res) => Ok(Json(TimeTrackDto::from_time_track_with_project_name(
            res.0, res.1,
        ))),
        Err(err) => match err {
            TimeTrackError::NotFound => Err(status::Custom(
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

#[delete("/timetrack/<project_id>/<time_track_id>")]
pub async fn delete(
    time_track_service: &State<Arc<TimeTrackService>>,
    user: UserValidation,
    project_id: UuidValidation,
    time_track_id: UuidValidation,
) -> Result<status::NoContent, status::Custom<Json<ErrorResponse>>> {
    let user = &user.0;
    let project_id = project_id.0.to_string();
    let time_track_id = time_track_id.0.to_string();

    match time_track_service
        .delete(user, project_id, time_track_id)
        .await
    {
        Ok(_) => Ok(status::NoContent),
        Err(err) => match err {
            TimeTrackError::NotFound => Err(status::Custom(
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
