use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub error_message: String,
}
