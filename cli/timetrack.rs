use chrono::{DateTime, Local};
use colored::{Color, Colorize};
use sitt_api::{
    handlers::dtos::time_track_dtos::TimeTrackDto, models::time_track_model::TimeTrackStatus,
};
use toml::value::Datetime;

use crate::{
    config::Config,
    project::get_project_id_by_name,
    sitt_client,
    utils::{self, print_and_exit_on_error},
};

pub fn start_time_tracking(config: &Config, name: &str) {
    let project_id_result = get_project_id_by_name(config, name);
    let project_id = print_and_exit_on_error(project_id_result);

    let api_response = sitt_client::start_time_tracking(config, &project_id);
    let timetrack = utils::print_and_exit_on_error(api_response);

    print_time_track(&timetrack)
}

pub fn stop_time_tracking(config: &Config, name: &str) {
    let project_id_result = get_project_id_by_name(config, name);
    let project_id = print_and_exit_on_error(project_id_result);

    let api_response = sitt_client::stop_time_tracking(config, &project_id);
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
        timetrack.project_name, status_with_color, timetrack.started_at
    );

    if let Some(stopped_at) = timetrack.stopped_at {
        let local_stopped_at: DateTime<Local> = stopped_at.with_timezone(&Local);
        println!("STOPPED AT:   {}", local_stopped_at);
        println!("DURATION:     {}", timetrack.total_duration);
    }
}
