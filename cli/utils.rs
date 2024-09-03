use std::{fmt::Display, process::exit};

use inquire::validator::StringValidator;
use sitt_api::handlers::dtos::project_dtos::ProjectDto;

pub fn print_and_exit_on_error<T, E>(result: Result<T, E>) -> T
where
    E: Display,
{
    result.unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    })
}

