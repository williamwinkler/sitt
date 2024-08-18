use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    pub error_message: String,
}
