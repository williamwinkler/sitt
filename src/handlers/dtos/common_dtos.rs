use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use serde::Serialize;
use validator::ValidationErrors;



#[derive(Serialize, Debug)]
pub enum ErrorResponse {
    Simple {
        error: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },
    Validation(ValidationErrors),
}

impl ErrorResponse {
    pub fn validation_error(
        validation_errors: ValidationErrors,
    ) -> status::Custom<Json<ErrorResponse>> {
        status::Custom(
            Status::UnprocessableEntity,
            Json(ErrorResponse::Validation(validation_errors)),
        )
    }

    pub fn internal_server_error() -> status::Custom<Json<ErrorResponse>> {
        status::Custom(
            Status::InternalServerError,
            Json(ErrorResponse::Simple {
                error: String::from("An internal server error occurred"),
                details: None,
            }),
        )
    }

    pub fn custom(
        status: Status,
        error_message: &str,
        details: Option<&str>,
    ) -> status::Custom<Json<ErrorResponse>> {
        status::Custom(
            status,
            Json(ErrorResponse::Simple {
                error: error_message.to_string(),
                details: details.map(|d| d.to_string()),
            }),
        )
    }
}
