use super::super::dtos::common_dtos::ErrorResponse;
use rocket::request::FromParam;
use rocket::serde::json::Json;
use uuid::Uuid;

#[derive(Debug)]
pub struct ValidateUuid(pub Uuid);

#[rocket::async_trait]
impl<'r> FromParam<'r> for ValidateUuid {
    type Error = Json<ErrorResponse>;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        match Uuid::parse_str(param) {
            Ok(uuid) => Ok(ValidateUuid(uuid)),
            Err(_) => Err(Json(ErrorResponse::Simple {
                error: "ValidationError".to_string(),
                details: Some("Invalid id. It needs to be UUID v4".to_string()),
            })),
        }
    }
}
