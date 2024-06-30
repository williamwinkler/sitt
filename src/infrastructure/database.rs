use aws_config;
use aws_sdk_dynamodb::Client;

#[derive(Debug)]
pub struct Database {
    pub client: Client,
}

impl Database {
    pub async fn new() -> Self {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);

        Database { client: client }
    }
}
