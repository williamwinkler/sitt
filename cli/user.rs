use core::fmt;
use std::process::exit;

use colored::{Color, Colorize};
use inquire::{Confirm, Select, Text};
use sitt_api::{
    handlers::dtos::user_dtos::{CreateUserDto, UserDto},
    models::user_model::UserRole,
};

use crate::{config::Config, sitt_client, utils};

struct SelectUser {
    pub id: String,
    pub name: String,
}

impl fmt::Display for SelectUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn create_user(config: &Config) {
    let name = Text::new("Name of user:").prompt().unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        exit(1);
    });

    let role_choice = Select::new("Choose role:", vec!["USER", "ADMIN"])
        .prompt()
        .unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            exit(1);
        });

    let mut role = UserRole::User;

    if role_choice == "ADMIN" {
        role = UserRole::Admin;

        let confirm_admin_choice = Confirm::new("Are you sure you want to create an ADMIN user?")
            .prompt()
            .unwrap_or_else(|err| {
                eprintln!("Error: {}", err);
                exit(1);
            });

        if !confirm_admin_choice {
            exit(0)
        }
    }

    let create_user_dto = CreateUserDto { name, role };

    let api_response = sitt_client::create_user(config, &create_user_dto);

    let user = utils::print_and_exit_on_error(api_response);

    println!("User was successfully created! ✅");
    print_user(&user);
}

pub fn get_user(config: &Config) {
    let user = select_user(config, "get");

    let include_api_key = Confirm::new("Should the API key be included?")
        .prompt()
        .unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            exit(1);
        });

    let api_response = sitt_client::get_user(config, &user.id, include_api_key);
    let user = utils::print_and_exit_on_error(api_response);

    print_user(&user);
}

pub fn get_users(config: &Config) {
    let result = sitt_client::get_users(config);
    let users = utils::print_and_exit_on_error(result);
    users.iter().for_each(print_user);
}

pub fn delete_user(config: &Config) {
    let user = select_user(config, "delete");

    let confirm_deletion = Confirm::new(&format!(
        "Are you sure you want to delete {}?",
        user.name.color(Color::Yellow)
    ))
    .prompt()
    .unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        exit(1);
    });

    if !confirm_deletion {
        exit(0)
    }

    let api_response = sitt_client::delete_user(config, &user.id);
    utils::print_and_exit_on_error(api_response);

    println!(
        "User {} was successfully deleted! ✅",
        user.name.color(Color::Yellow)
    );
}

fn select_user(config: &Config, action: &str) -> SelectUser {
    let result = sitt_client::get_users(config);
    let users = utils::print_and_exit_on_error(result);

    let select_user_options: Vec<SelectUser> = users
        .iter()
        .map(|user| SelectUser {
            id: user.id.clone(),
            name: user.name.clone(),
        })
        .collect();

    let chosen_user = Select::new(
        &format!("Which user would you like to {}?:", action),
        select_user_options,
    )
    .prompt()
    .unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        exit(1)
    });

    chosen_user
}

fn print_user(user: &UserDto) {
    let api_key = if let Some(api_key) = user.api_key.clone() {
        api_key
    } else {
        String::from("******")
    };

    println!(
        r#"ID:          {}
NAME:        {}
ROLE:        {}
API_KEY:     {}
CREATED AT:  {}
CREATED BY:  {}
    "#,
        user.id,
        user.name.color(Color::Yellow),
        user.role,
        api_key,
        user.created_at,
        user.created_by,
    )
}
