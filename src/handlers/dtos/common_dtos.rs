use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn custom(status: Status, error_message: &str) -> status::Custom<Json<ErrorResponse>> {
        status::Custom(
            status,
            Json(ErrorResponse {
                error: error_message.to_string(),
            }),
        )
    }

    pub fn internal_server_error() -> status::Custom<Json<ErrorResponse>> {
        status::Custom(
            Status::InternalServerError,
            Json(ErrorResponse {
                error: String::from("An internal server error occurred"),
            }),
        )
    }
}
