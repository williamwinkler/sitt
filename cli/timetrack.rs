use std::{process::exit, time::Duration};

use chrono::{DateTime, Local, Utc};
use colored::{Color, Colorize};
use inquire::{Confirm, Select, Text};
use sitt_api::{
    handlers::dtos::time_track_dtos::{CreateTimeTrackDto, TimeTrackDto},
    models::time_track_model::TimeTrackStatus,
};

use crate::{
    config::Config,
    project::{get_project_id_by_name, resolve_project_name, ProjectSelectOption},
    sitt_client, timetrack,
    utils::{self, print_and_exit_on_error, DATETIME_FORMAT},
    NameArg, StartArgs,
};

use std::fmt;

struct CliTimeTrack {
    pub id: String,
    pub project_id: String,
    pub status: TimeTrackStatus,
    pub comment: Option<String>,
    pub started_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub total_duration: String,
}

impl fmt::Display for CliTimeTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let comment = if self.comment.is_none() {
            "".to_string()
        } else {
            format!("- {}", self.comment.clone().unwrap().color(Color::Green))
        };

        if let Some(stopped_at) = self.stopped_at {
            write!(
                f,
                "{} -> {} | {}  {}",
                self.started_at
                    .with_timezone(&Local)
                    .format(DATETIME_FORMAT),
                stopped_at.with_timezone(&Local).format(DATETIME_FORMAT),
                self.total_duration,
                comment
            )
        } else {
            write!(
                f,
                "{} ->     IN PROGRESS     | {} ⏱️  {}",
                self.started_at
                    .with_timezone(&Local)
                    .format(DATETIME_FORMAT),
                self.total_duration,
                comment
            )
        }
    }
}

impl From<TimeTrackDto> for CliTimeTrack {
    fn from(dto: TimeTrackDto) -> Self {
        CliTimeTrack {
            id: dto.time_track_id,
            project_id: dto.project_id,
            status: dto.status,
            comment: dto.comment,
            started_at: dto.started_at,
            stopped_at: dto.stopped_at,
            total_duration: dto.total_duration,
        }
    }
}

pub fn start_time_tracking(config: &Config, args: &StartArgs) {
    let name = resolve_project_name(
        args.name.clone(),
        config,
        "start tracking on",
        ProjectSelectOption::InActive,
    );
    let project_id_result = get_project_id_by_name(config, &name);
    let project_id = print_and_exit_on_error(project_id_result);

    let mut comment = args.comment.clone(); // Clone the Option<String> from args

    if comment.is_none() {
        comment = ask_for_comment(None);
    }

    let api_response = sitt_client::start_time_tracking(config, &project_id, comment);
    let timetrack = utils::print_and_exit_on_error(api_response);

    print_time_track_full(&timetrack)
}

pub fn stop_time_tracking(config: &Config, project_args: &NameArg) {
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

    print_time_track_full(&timetrack)
}

pub fn add_time_tracking(config: &Config, args: &NameArg) {
    let name = resolve_project_name(
        args.name.clone(),
        config,
        "add time on",
        ProjectSelectOption::None,
    );
    let project_id_result = get_project_id_by_name(config, &name);
    let project_id = print_and_exit_on_error(project_id_result);

    let started_at = utils::prompt_user_for_datetime(
        &format!("Enter the {} date", "starting".color(Color::Yellow)),
        None,
        None,
    );

    let stopped_at = utils::prompt_user_for_datetime(
        &format!("Enter the {} date", "stopping".color(Color::Yellow),),
        Some(started_at),
        Some(started_at),
    );

    let comment = ask_for_comment(None);

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
    .unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        exit(1);
    });

    if !confirm_choice {
        exit(0)
    }

    let create_time_track = CreateTimeTrackDto {
        project_id,
        started_at,
        stopped_at,
        comment,
    };

    let api_response = sitt_client::add_time_tracking(config, &create_time_track);
    let timetrack = utils::print_and_exit_on_error(api_response);

    print_time_track_full(&timetrack)
}

pub fn get_time_trackings(config: &Config, args: &NameArg) {
    let name = resolve_project_name(
        args.name.clone(),
        config,
        "fetching logged time on",
        ProjectSelectOption::None,
    );
    let project_id_result = get_project_id_by_name(config, &name);
    let project_id = print_and_exit_on_error(project_id_result);

    let api_response = sitt_client::get_time_trackings(config, &project_id);
    let timetrack_list = utils::print_and_exit_on_error(api_response);

    if timetrack_list.is_empty() {
        println!(
            "You have not yet tracked any time on {}",
            name.color(Color::Cyan)
        );
        exit(0)
    }

    println!(
        "You have logged time {} times on {}:\n",
        timetrack_list.len().to_string().color(Color::Yellow),
        name.color(Color::Cyan)
    );

    timetrack_list
        .iter()
        .for_each(|t| println!("{}", CliTimeTrack::from(t.clone())));
}

pub fn edit_time_track(config: &Config, args: &NameArg) {
    let name = resolve_project_name(
        args.name.clone(),
        config,
        "update time on",
        ProjectSelectOption::None,
    );
    let project_id_result = get_project_id_by_name(config, &name);
    let project_id = print_and_exit_on_error(project_id_result);

    let time_track = select_time_track(config, "update", &name, &project_id);

    if time_track.status == TimeTrackStatus::InProgress {
        println!("You can't edit a time log which is in progress. 🚫");
        exit(0);
    }

    let started_at = utils::prompt_user_for_datetime(
        &format!("Enter the {} date", "starting".color(Color::Yellow)),
        None,
        Some(time_track.started_at),
    );

    // Determine the placeholder for the new stopped_at based on the relation to the previous stopped_at
    let stopped_at_placeholder = match time_track.stopped_at {
        Some(stopped_at) if started_at > stopped_at => started_at,
        Some(stopped_at) => stopped_at,
        None => started_at, // or a default value that makes sense in your context
    };

    let stopped_at = utils::prompt_user_for_datetime(
        &format!("Enter the {} date", "stopping".color(Color::Yellow),),
        Some(started_at),
        Some(stopped_at_placeholder),
    );

    let duration = {
        let time_delta = stopped_at - started_at;
        Duration::new(time_delta.num_seconds() as u64, 0)
    };

    let comment = ask_for_comment(time_track.comment);

    let confirm_choice = Confirm::new(&format!(
        "Are you sure, you want to edit the logged time to {} on project {}?",
        humantime::format_duration(duration)
            .to_string()
            .color(Color::Yellow),
        name.color(Color::Cyan)
    ))
    .prompt()
    .unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        exit(1);
    });

    if !confirm_choice {
        exit(0)
    }

    let update_time_track = CreateTimeTrackDto {
        project_id,
        started_at,
        stopped_at,
        comment,
    };

    let api_response = sitt_client::update_time_track(config, &time_track.id, &update_time_track);
    utils::print_and_exit_on_error(api_response);

    println!("The time log was successfully updated! ✅")
}

pub fn delete_time_tracking(config: &Config, args: &NameArg) {
    let name = resolve_project_name(
        args.name.clone(),
        config,
        "delete time on",
        ProjectSelectOption::None,
    );

    let project_id_result = get_project_id_by_name(config, &name);
    let project_id = print_and_exit_on_error(project_id_result);

    let time_track = select_time_track(config, "delete", &name, &project_id);

    let confirm_deletion = Confirm::new("Are you sure you want to delete?")
        .prompt()
        .unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            exit(1)
        });

    if !confirm_deletion {
        exit(0)
    }

    let api_response =
        sitt_client::delete_time_track(config, &time_track.project_id, &time_track.id);
    utils::print_and_exit_on_error(api_response);

    println!("The time log was successfully deleted! ✅")
}

fn select_time_track(
    config: &Config,
    action: &str,
    project_name: &str,
    project_id: &str,
) -> CliTimeTrack {
    let api_response = sitt_client::get_time_trackings(config, project_id);
    let timetrack_list = utils::print_and_exit_on_error(api_response);

    if timetrack_list.is_empty() {
        println!(
            "You have not yet tracked any time on {}",
            project_name.color(Color::Cyan)
        );
        exit(0)
    }

    let cli_timetrack_list: Vec<CliTimeTrack> = timetrack_list
        .iter()
        .map(|t| CliTimeTrack::from(t.clone()))
        .collect();

    let chosen_time_track = Select::new(
        &format!("Which logged time would you like to {}?:", action),
        cli_timetrack_list,
    )
    .prompt()
    .unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        exit(1)
    });

    chosen_time_track
}

fn ask_for_comment(initial_value: Option<String>) -> Option<String> {
    let comment = Text::new("📝 Add comment:")
        .with_help_message("If no comment, leave blank and press Enter")
        .with_initial_value(&initial_value.unwrap_or_else(|| "".to_string()))
        .prompt()
        .unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            exit(1);
        });

    if comment.is_empty() {
        None
    } else {
        Some(comment)
    }
}

fn print_time_track_full(timetrack: &TimeTrackDto) {
    let status_with_color = {
        let mut status_with_color = timetrack.status.to_string().color(Color::Yellow);
        if timetrack.status == TimeTrackStatus::InProgress {
            status_with_color = (timetrack.status.to_string() + " ⏱️").color(Color::BrightGreen);
        }
        status_with_color
    };

    println!(
        r#"
PROJECT:      {}
STATUS:       {}
COMMENT:      {}
STARTED AT:   {}"#,
        timetrack.project_name.color(Color::Cyan),
        status_with_color,
        timetrack
            .comment
            .clone()
            .unwrap_or("".to_string())
            .color(Color::Green),
        timetrack
            .started_at
            .with_timezone(&Local)
            .format(DATETIME_FORMAT),
    );

    if let Some(stopped_at) = timetrack.stopped_at {
        println!(
            "STOPPED AT:   {}",
            stopped_at.with_timezone(&Local).format(DATETIME_FORMAT)
        );
        println!("DURATION:     {}", timetrack.total_duration);
    }
}
