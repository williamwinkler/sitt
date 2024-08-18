use sitt_api::handlers::dtos::project_dtos::CreateProjectDto;

use crate::{config::Config, sitt_client, utils::print_and_exit_on_error};

pub fn create_project(config: &Config, name: String) {
    let create_project_dto = CreateProjectDto { name: name };

    let result = sitt_client::create_project(&config, &create_project_dto);
    let project = print_and_exit_on_error(result);

    let project_json = serde_json::to_string_pretty(&project).unwrap();
    println!("{}", project_json);
}

pub fn get_projects(config: &Config) {
    let result = sitt_client::get_projects(config);
    let projects = print_and_exit_on_error(result);

    let projects_json = serde_json::to_string_pretty(&projects).unwrap();
    println!("{}", projects_json);
}

// pub fn delete_project(config: &Config, name: String) {
//   // find project with name in cache

//   let result = sitt_client::create_project(&config);
//   let project = handle_error(result);

//     let project_json = serde_json::to_string_pretty(&project).unwrap();
//     println!("{}", project_json);
// }
