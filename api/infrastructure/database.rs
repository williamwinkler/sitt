use aws_config;
use aws_sdk_dynamodb::Client;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("The item was not found")]
    NotFound,
    #[error("Item from table '{table}' failed to be converted for id: {id}")]
    Convertion { table: String, id: String },
    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[derive(Debug)]
pub struct Database {
    pub client: Client,
}

impl Database {
    pub async fn new() -> Self{
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        Database { client }
    }
}
