use super::super::dtos::common_dtos::ErrorResponse;
use rocket::http::Status;
use rocket::request::FromParam;
use rocket::response::status;
use rocket::serde::json::Json;
use uuid::Uuid;

#[derive(Debug)]
pub struct ValidateUuid(pub Uuid);

#[rocket::async_trait]
impl<'r> FromParam<'r> for ValidateUuid {
    type Error = status::Custom<Json<ErrorResponse>>;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        match Uuid::parse_str(param) {
            Ok(uuid) => Ok(ValidateUuid(uuid)),
            Err(err) => Err(status::Custom(
                Status::UnprocessableEntity,
                Json(ErrorResponse {
                    error_mesage: err.to_string(),
                }),
            )),
        }
    }
}
