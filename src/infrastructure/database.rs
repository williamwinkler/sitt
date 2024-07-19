use aws_config;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::Error;
use aws_smithy_runtime_api::client::result::SdkError;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::http::Response;
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("The item was not found")]
    NotFound,
    #[error("An item with {key} already exists with value: {value}")]
    AlreadyExists { key: String, value: String },
    #[error("Item from table '{table}' failed to be converted for id: {id}")]
    Convertion { table: String, id: String},
    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[derive(Debug)]
pub struct Database {
    pub client: Client,
}

impl Database {
    pub async fn new() -> Self {
        let required_vars = ["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY", "AWS_REGION"];
        for var in required_vars {
            if env::var(var).is_err() {
                panic!("Environment variable '{}' is not set", var);
            }
        }

        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);

        Database { client: client }
    }
}
