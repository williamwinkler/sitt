use colored::{Color, Colorize};
use etcetera::BaseStrategy;
use inquire::{validator::Validation, Text};
use serde::{Deserialize, Serialize};
use std::{fs, io, process::exit};
use thiserror::Error;
use url::Url;

const CONFIG_FILE: &str = "sitt.toml";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing file or failed to open it. Expected it at path: {0}")]
    MissingFile(io::Error),
    #[error("The configuration file is invalid: {0}")]
    InvalidConfig(String),
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    api_key: String,
    sitt_url: String,
}

impl Config {
    fn new(api_key: String, sitt_url: String) -> Self {
        Self { api_key, sitt_url }
    }

    pub fn load() -> Result<Self, ConfigError> {
        let config_path = etcetera::choose_base_strategy()
            .unwrap_or_else(|err| {
                eprintln!("Error: {}", err);
                exit(1);
            })
            .config_dir()
            .join(CONFIG_FILE);

        let config_content = fs::read_to_string(config_path).map_err(ConfigError::MissingFile)?;

        let config: Config = toml::from_str(&config_content)
            .map_err(|err| ConfigError::InvalidConfig(err.to_string()))?;

        Ok(config)
    }

    pub fn setup() -> Self {
        println!("Setup configuration:");

        let api_key_validator = |input: &str| {
            if input.chars().count() != 32 {
                Ok(Validation::Invalid("Invalid API key, try again".into()))
            } else {
                Ok(Validation::Valid)
            }
        };

        let url_validator = |input: &str| match Url::parse(input) {
            Ok(s) => {
                println!("{}", s);
                Ok(Validation::Valid)
            }
            Err(_) => Ok(Validation::Invalid("It must be a valid URL".into())),
        };

        let sitt_url = Text::new(&format!("{} URL:", "sitt".color(Color::Yellow)))
            .with_help_message(&format!(
                "The URL where the {} API is hosted",
                "sitt".color(Color::Yellow)
            ))
            .with_validator(url_validator)
            .prompt()
            .unwrap_or_else(|err| {
                eprintln!("Error: {}", err);
                exit(1);
            });

        let api_key = Text::new(&format!("{} API key:", "sitt".color(Color::Yellow)))
            .with_validator(api_key_validator)
            .prompt()
            .unwrap_or_else(|err| {
                eprintln!("Error: {}", err);
                exit(1);
            });

        let config = Config::new(api_key, sitt_url);
        let toml = toml::to_string(&config).unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            exit(1);
        });

        // Get configuration path fiting the OS
        let config_path = etcetera::choose_base_strategy()
            .unwrap_or_else(|err| {
                eprintln!("Error: {}", err);
                exit(1);
            })
            .config_dir()
            .join(CONFIG_FILE);

        fs::write(&config_path, toml).unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            exit(1);
        });

        println!("\nConfiguration was successful ✅");
        println!(
            "Configuration saved at: {}",
            &config_path.to_string_lossy().to_string()
        );

        config
    }

    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }
    pub fn get_url(&self) -> &str {
        &self.sitt_url
    }
}
