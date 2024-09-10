use std::{process::exit, time::Duration};

use chrono::{DateTime, Local};
use colored::{Color, Colorize};
use inquire::Confirm;
use sitt_api::{
    handlers::dtos::time_track_dtos::{CreateTimeTrackDto, TimeTrackDto},
    models::time_track_model::TimeTrackStatus,
};

use crate::{
    config::Config,
    project::{get_project_id_by_name, resolve_project_name, ProjectSelectOption},
    sitt_client,
    utils::{self, print_and_exit_on_error},
    ProjectArgs,
};

pub fn start_time_tracking(config: &Config, project_args: &ProjectArgs) {
    let name = resolve_project_name(
        project_args.name.clone(),
        config,
        "start tracking on",
        ProjectSelectOption::InActive,
    );
    let project_id_result = get_project_id_by_name(config, &name);
    let project_id = print_and_exit_on_error(project_id_result);

    let api_response = sitt_client::start_time_tracking(config, &project_id);
    let timetrack = utils::print_and_exit_on_error(api_response);

    print_time_track(&timetrack)
}

pub fn stop_time_tracking(config: &Config, project_args: &ProjectArgs) {
    let name = resolve_project_name(
        project_args.name.clone(),
        config,
        "stop tracking on",
        ProjectSelectOption::Active,
    );
    let project_id_result = get_project_id_by_name(config, &name);
    let project_id = print_and_exit_on_error(project_id_result);

    let api_response = sitt_client::stop_time_tracking(config, &project_id);
    let timetrack = utils::print_and_exit_on_error(api_response);

    print_time_track(&timetrack)
}

pub fn add_time_tracking(config: &Config, args: &ProjectArgs) {
    let name = resolve_project_name(
        args.name.clone(),
        config,
        "add time on",
        ProjectSelectOption::None,
    );
    let project_id_result = get_project_id_by_name(config, &name);
    let project_id = print_and_exit_on_error(project_id_result);

    let started_at = utils::prompt_user_for_datetime(&format!(
        "Enter the {} date",
        "starting".color(Color::Yellow)
    ));
    let stopped_at = utils::prompt_user_for_datetime(&format!(
        "Enter the {} date",
        "stopping".color(Color::Yellow)
    ));

    let duration = {
        let time_delta = stopped_at - started_at;
        Duration::new(time_delta.num_seconds() as u64, 0)
    };

    let confirm_choice = Confirm::new(&format!(
        "Are you sure, you want to add {} to project {}?",
        humantime::format_duration(duration)
            .to_string()
            .color(Color::Yellow),
        name.color(Color::Cyan)
    ))
    .prompt()
    .expect("Failed prompting user for add confirmation");

    if !confirm_choice {
        exit(0)
    }

    let create_time_track = CreateTimeTrackDto {
        project_id,
        started_at,
        stopped_at,
    };

    let api_response = sitt_client::add_time_tracking(config, &create_time_track);
    let timetrack = utils::print_and_exit_on_error(api_response);

    print_time_track(&timetrack)
}

fn print_time_track(timetrack: &TimeTrackDto) {
    let status_with_color = {
        let mut status_with_color = timetrack.status.to_string().color(Color::Yellow);
        if timetrack.status == TimeTrackStatus::InProgress {
            status_with_color = (timetrack.status.to_string() + " ⏱️").color(Color::BrightGreen);
        }
        status_with_color
    };
    let local_created_at: DateTime<Local> = timetrack.started_at.with_timezone(&Local);

    println!(
        r#"
NAME:         {}
STATUS:       {}
STARTED AT:   {}"#,
        timetrack.project_name.color(Color::Cyan),
        status_with_color,
        local_created_at
    );

    if let Some(stopped_at) = timetrack.stopped_at {
        let local_stopped_at: DateTime<Local> = stopped_at.with_timezone(&Local);
        println!("STOPPED AT:   {}", local_stopped_at);
        println!("DURATION:     {}", timetrack.total_duration);
    }
}
