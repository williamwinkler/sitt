use crate::config::Config;
use reqwest::{
    self,
    blocking::Client,
    header::{HeaderMap, HeaderValue},
};
use sitt_api::handlers::dtos::{
    common_dtos::ErrorResponse,
    project_dtos::{CreateProjectDto, ProjectDto},
    time_track_dtos::TimeTrackDto,
};
use std::{io::Read, time::Duration};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized request")]
    Unauthorized,
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error("Failed building request: {0}")]
    BuildRequest(String),
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Failed to parse response body: {0}")]
    ParseResponseBodyFailed(String),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

const PROJECTS_PATH: &str = "/api/v1/projects";
const TIME_TRACKS_PATH: &str = "/api/v1/timetrack";
const USERS_PATH: &str = "/api/v1/users";

struct ApiClient {
    client: Client,
    base_url: Url,
}

impl ApiClient {
    pub fn build(config: &Config) -> Result<Self, ClientError> {
        // Get API key
        let api_key = HeaderValue::from_str(config.get_api_key())
            .map_err(|err| ClientError::BuildRequest(err.to_string()))?;

        // Create header with API key
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", api_key);

        let client = reqwest::blocking::ClientBuilder::new()
            .default_headers(headers)
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(|err| ClientError::BuildRequest(err.to_string()))?;

        let base_url = Url::parse(config.get_url())
            .map_err(|_| ClientError::BuildRequest("Failed to create base_url".to_string()))?;

        Ok(Self { client, base_url })
    }

    pub fn build_url(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        url.set_path(path);
        url
    }

    fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        mut response: reqwest::blocking::Response,
    ) -> Result<T, ClientError> {
        match response.status() {
            reqwest::StatusCode::OK
            | reqwest::StatusCode::CREATED
            | reqwest::StatusCode::NO_CONTENT => {
                // If the response has no body (e.g., 204 No Content), return unit type `()`
                if response.status() == reqwest::StatusCode::NO_CONTENT {
                    return Ok((serde_json::from_str("null").unwrap()));
                }

                // Otherwise, attempt to deserialize the body into the desired type `T`
                response
                    .json()
                    .map_err(|err| ClientError::ParseResponseBodyFailed(err.to_string()))
            }
            reqwest::StatusCode::BAD_REQUEST => {
                let error_response: ErrorResponse = response
                    .json()
                    .map_err(|err| ClientError::ParseResponseBodyFailed(err.to_string()))?;
                Err(ClientError::BadRequest(error_response.error_message))
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(ClientError::Unauthorized),
            reqwest::StatusCode::NOT_FOUND => {
                let error_response: ErrorResponse = response
                    .json()
                    .map_err(|err| ClientError::ParseResponseBodyFailed(err.to_string()))?;
                Err(ClientError::NotFound(error_response.error_message))
            }
            reqwest::StatusCode::CONFLICT => {
                let error_response: ErrorResponse = response
                    .json()
                    .map_err(|err| ClientError::ParseResponseBodyFailed(err.to_string()))?;
                Err(ClientError::Conflict(error_response.error_message))
            }
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
                let error_response: ErrorResponse = response
                    .json()
                    .map_err(|err| ClientError::ParseResponseBodyFailed(err.to_string()))?;
                Err(ClientError::InternalServerError(
                    error_response.error_message,
                ))
            }
            _ => {
                let error_message = response
                    .text()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(ClientError::RequestFailed(error_message))
            }
        }
    }
}

pub fn create_project(
    config: &Config,
    create_project_dto: &CreateProjectDto,
) -> Result<ProjectDto, ClientError> {
    let api = ApiClient::build(config)?;
    let url = api.build_url(PROJECTS_PATH);

    let response = api
        .client
        .post(url)
        .json(create_project_dto)
        .send()
        .map_err(|err| ClientError::BuildRequest(err.to_string()))?;

    let project = api.handle_response::<ProjectDto>(response)?;

    Ok(project)
}

pub fn get_project_by_id(config: &Config, project_id: &str) -> Result<ProjectDto, ClientError> {
    let api = ApiClient::build(config)?;
    let url = api.build_url(&format!("{}/{}", PROJECTS_PATH, project_id.clone()));

    let response = api.client.get(url).send()?;

    let project = api.handle_response::<ProjectDto>(response)?;

    Ok(project)
}

pub fn get_projects(config: &Config) -> Result<Vec<ProjectDto>, ClientError> {
    let api = ApiClient::build(config)?;
    let url = api.build_url(PROJECTS_PATH);

    let response = api.client.get(url).send()?;

    let projects = api.handle_response::<Vec<ProjectDto>>(response)?;

    Ok(projects)
}

pub fn update_project(
    config: &Config,
    project_id: &str,
    update_project_dto: &CreateProjectDto,
) -> Result<ProjectDto, ClientError> {
    let api = ApiClient::build(config)?;
    let url = api.build_url(&format!("{}/{}", PROJECTS_PATH, project_id.clone()));

    let response = api
        .client
        .put(url)
        .json(update_project_dto)
        .send()
        .map_err(|err| ClientError::BuildRequest(err.to_string()))?;

    let project = api.handle_response(response)?;

    Ok(project)
}

pub fn delete_project(config: &Config, project_id: &str) -> Result<(), ClientError> {
    let api = ApiClient::build(config)?;
    let path = &format!("{}/{}", PROJECTS_PATH, project_id);
    let url = api.build_url(path);

    let response = api.client.delete(url).send()?;

    api.handle_response::<()>(response)?;

    Ok(())
}

pub fn start_time_tracking(config: &Config, project_id: &str) -> Result<TimeTrackDto, ClientError> {
    let api = ApiClient::build(config)?;
    let path = &format!("{}/{}/start", TIME_TRACKS_PATH, project_id);
    let url = api.build_url(path);

    println!("{url}");

    let response = api.client.post(url).send()?;

    let timetrack = api.handle_response::<TimeTrackDto>(response)?;

    Ok(timetrack)
}

pub fn stop_time_tracking(config: &Config, project_id: &str) -> Result<TimeTrackDto, ClientError> {
    let api = ApiClient::build(config)?;
    let path = &format!("{}/{}/stop", TIME_TRACKS_PATH, project_id);
    let url = api.build_url(path);

    let response = api.client.post(url).send()?;

    let timetrack = api.handle_response::<TimeTrackDto>(response)?;

    Ok(timetrack)
}