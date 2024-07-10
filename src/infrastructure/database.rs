use aws_config;
use aws_sdk_dynamodb::Client;
use std::env;

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
