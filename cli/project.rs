use crate::{
    config::Config,
    sitt_client,
    utils::{self, print_and_exit_on_error, DATETIME_FORMAT},
    NameArg,
};
use chrono::Local;
use colored::{Color, Colorize};
use etcetera::{self, BaseStrategy};
use inquire::{validator::Validation, Confirm, Select, Text};
use serde::{Deserialize, Serialize};
use sitt_api::{
    handlers::dtos::project_dtos::{CreateProjectDto, ProjectDto},
    models::project_model::ProjectStatus,
};
use std::{fs, path::PathBuf, process::exit};
use thiserror::Error;

const CACHE_FILE: &str = "sitt-projects.toml";

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("No project named: {0}")]
    NoProjectWithName(String),
    #[error("Failed finding ID for project {0} in cache")]
    CacheError(String),
}

pub enum ProjectSelectOption {
    None,
    Active,
    InActive,
}

#[derive(Serialize, Deserialize)]
struct ProjectCache {
    id: String,
    name: String,
}

pub fn create_project(config: &Config, args: NameArg) {
    let name = if let Some(name) = args.name {
        name
    } else {
        let name = Text::new("Project name:").prompt().unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            exit(1);
        });

        name
    };
    let create_project_dto = CreateProjectDto { name };

    let result = sitt_client::create_project(config, &create_project_dto);
    let project = utils::print_and_exit_on_error(result);

    println!("New project created ✅:");
    print_project(&project);
}

pub fn get_project_by_name(config: &Config, args: &NameArg) {
    let name = resolve_project_name(args.name.clone(), config, "get", ProjectSelectOption::None);

    let project_id_result = get_project_id_by_name(config, &name);
    let project_id = print_and_exit_on_error(project_id_result);

    let api_response = sitt_client::get_project_by_id(config, &project_id);
    let project = utils::print_and_exit_on_error(api_response);

    print_project(&project);
}

pub fn update_project(config: &Config, args: &NameArg) {
    let name = resolve_project_name(
        args.name.clone(),
        config,
        "update",
        ProjectSelectOption::None,
    );

    let project_id_result = get_project_id_by_name(config, &name);
    let project_id = print_and_exit_on_error(project_id_result);

    let length_validator = |input: &str| {
        if input.chars().count() == 0 {
            Ok(Validation::Invalid("You have to enter something.".into()))
        } else if input.chars().count() > 25 {
            Ok(Validation::Invalid("Too long.".into()))
        } else {
            Ok(Validation::Valid)
        }
    };

    let new_name = Text::new("New project name:")
        .with_initial_value(&name)
        .with_validator(length_validator)
        .prompt()
        .unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            exit(1);
        });

    let update_project_dto = CreateProjectDto { name: new_name };

    let api_response = sitt_client::update_project(config, &project_id, &update_project_dto);
    let project = utils::print_and_exit_on_error(api_response);

    print_project(&project);
}

pub fn delete_project(config: &Config, args: &NameArg) {
    let name = resolve_project_name(
        args.name.clone(),
        config,
        "delete",
        ProjectSelectOption::None,
    );

    let project_id_result = get_project_id_by_name(config, &name);
    let project_id = print_and_exit_on_error(project_id_result);

    let confirm_deletion = Confirm::new("Are you sure you want to delete?")
        .prompt()
        .unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            exit(1);
        });

    if !confirm_deletion {
        exit(0)
    }

    let api_response = sitt_client::delete_project(config, &project_id);

    utils::print_and_exit_on_error(api_response);

    // Recache projects
    recache_projects(config);

    println!(
        "Project {} was successfully deleted! ✅",
        name.color(Color::Cyan)
    );
}

pub fn get_projects(config: &Config) {
    let result = sitt_client::get_projects(config);
    let projects = utils::print_and_exit_on_error(result);

    if !projects.is_empty() {
        println!("Your {} projects: ", projects.len());
        projects.iter().for_each(print_project);
    } else {
        println!("You have no projects");
    }
}

pub fn select_project(config: &Config, action: &str, select_option: ProjectSelectOption) -> String {
    let result = sitt_client::get_projects(config);
    let projects = utils::print_and_exit_on_error(result);

    let mut options: Vec<&str> = Vec::new();
    match select_option {
        ProjectSelectOption::None => options = projects.iter().map(|p| p.name.as_str()).collect(),
        ProjectSelectOption::Active => {
            options = projects
                .iter()
                .filter(|p| p.status == ProjectStatus::Active)
                .map(|p| p.name.as_str())
                .collect()
        }
        ProjectSelectOption::InActive => {
            options = projects
                .iter()
                .filter(|p| p.status == ProjectStatus::Inactive)
                .map(|p| p.name.as_str())
                .collect()
        }
    }

    if options.is_empty() {
        println!("No projects to {} 👀", action);
        exit(0);
    }

    let project_name = Select::new(
        &format!("Which project do you want to {}?:", action),
        options,
    )
    .prompt()
    .unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        exit(1);
    });

    project_name.to_string()
}

fn print_project(project: &ProjectDto) {
    let status_with_color = {
        let mut status_with_color = project.status.to_string().color(Color::Yellow);
        if project.status == ProjectStatus::Active {
            status_with_color = (project.status.to_string() + " ⏱️").color(Color::BrightGreen);
        }
        status_with_color
    };

    println!(
        r#"
NAME:         {}
STATUS:       {}
TIME LOGGED:  {}
CREATED AT:   {}"#,
        project.name.color(Color::Cyan),
        status_with_color,
        project.total_duration,
        project
            .created_at
            .with_timezone(&Local)
            .format(DATETIME_FORMAT),
    );

    if let Some(modified_at) = project.modified_at {
        println!(
            "MODIFIED AT:  {}",
            modified_at.with_timezone(&Local).format(DATETIME_FORMAT)
        )
    }
}

pub fn resolve_project_name(
    args_name: Option<String>,
    config: &Config,
    action: &str,
    option: ProjectSelectOption,
) -> String {
    // If `args.name` is provided, return it; otherwise prompt the user to select a project
    if let Some(project_name) = args_name {
        project_name
    } else {
        select_project(config, action, option)
    }
}

pub fn get_project_id_by_name(config: &Config, name: &str) -> Result<String, ProjectError> {
    // Load cache of projects
    let cache_file_path = etcetera::choose_base_strategy()
        .unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            exit(1);
        })
        .cache_dir()
        .join(CACHE_FILE);

    let mut cache: Vec<ProjectCache> = Vec::new();

    // If no cache file exists, we need to create it to reduce API calls
    if !cache_file_path.exists() {
        let new_cache_result = cache_projects(config, &cache_file_path);
        let new_cache = print_and_exit_on_error(new_cache_result);

        // Add the newly cached projects to the existing project cache
        cache.extend(new_cache);
    } else {
        let cache_content = fs::read_to_string(&cache_file_path).unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            exit(1);
        });

        let old_project_cache_result = serde_json::from_str(&cache_content);

        // Check if the caches was deserialized succesfully
        let mut project_cache = match old_project_cache_result {
            Ok(existing_cache) => existing_cache, // If deserialization is successful, use the existing cache
            Err(_) => {
                // If deserialization fails, create a new cache
                let new_project_cache_result = cache_projects(config, &cache_file_path);

                print_and_exit_on_error(new_project_cache_result) // Use the newly created cache
            }
        };

        // Check if the project name is in the cache => if not update cache
        let project = project_cache.iter().find(|p| p.name == name);
        if project.is_none() {
            let new_project_cache_result = cache_projects(config, &cache_file_path);
            project_cache = print_and_exit_on_error(new_project_cache_result);
        }

        cache.extend(project_cache);
    }

    let project_id = cache
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.id.clone())
        .ok_or_else(|| ProjectError::NoProjectWithName(name.to_string()));

    project_id
}

fn recache_projects(config: &Config) {
    let cache_file_path = etcetera::choose_base_strategy()
        .unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            exit(1);
        })
        .cache_dir()
        .join(CACHE_FILE);

    let cache_result = cache_projects(config, &cache_file_path);
    print_and_exit_on_error(cache_result);
}

// Cachce projects with project_id & name
fn cache_projects(
    config: &Config,
    cache_file_path: &PathBuf,
) -> Result<Vec<ProjectCache>, ProjectError> {
    let api_response = sitt_client::get_projects(config);
    let fetched_projects = print_and_exit_on_error(api_response);

    let new_project_cache: Vec<ProjectCache> = fetched_projects
        .iter()
        .map(|project| ProjectCache {
            id: project.project_id.clone(),
            name: project.name.clone(),
        })
        .collect();

    let serialized_cache = serde_json::to_string_pretty(&new_project_cache)
        .map_err(|err| ProjectError::CacheError(err.to_string()))?;

    fs::write(cache_file_path, serialized_cache).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        exit(1);
    });

    Ok(new_project_cache)
}
