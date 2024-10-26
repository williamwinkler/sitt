use std::{fmt::Display, process::exit, time::Duration};

use chrono::{
    DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc,
};
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{DateSelect, Text};

pub const DATETIME_FORMAT: &str = "%d/%m/%Y %H:%M:%S";

pub fn print_and_exit_on_error<T, E>(result: Result<T, E>) -> T
where
    E: Display,
{
    result.unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    })
}

pub fn get_spinner(msg: String) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_message(msg);
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ") // Loading animation
            .template("{spinner} {msg}")
            .unwrap(),
    );

    spinner
}

pub fn prompt_user_for_datetime(
    msg: &str,
    min_date: Option<DateTime<Utc>>,
    placeholder_date: Option<DateTime<Utc>>,
) -> DateTime<Utc> {
    // Ask the user to select a date (in local time zone)
    let mut date = DateSelect::new(msg);
    let mut initial_timestamp = String::from("12:00:00");
    // Ask for time (HH:MM:SS) in local time zone
    let mut time_input = Text::new("Enter the time (HH:MM:SS in 24-hour format):")
        .with_help_message("Example: 14:30:15");

    // Set min date and placeholder date and time if present
    if let Some(min_date) = min_date {
        let naive_min_date = get_local_naive_date_from_utc_datetime(min_date);
        date = date.with_min_date(naive_min_date);

        initial_timestamp = get_local_time_as_str(min_date);
    }
    if let Some(placeholder_date) = placeholder_date {
        let naive_placeholder_date = get_local_naive_date_from_utc_datetime(placeholder_date);
        date = date.with_starting_date(naive_placeholder_date);

        initial_timestamp = get_local_time_as_str(placeholder_date);
    }

    time_input = time_input.with_initial_value(&initial_timestamp);

    let date = date.prompt().unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        exit(1);
    });

    let time_input_str = time_input.prompt().unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        exit(1);
    });

    // Parse the time (in local time zone) and handle errors
    let time = NaiveTime::parse_from_str(&time_input_str, "%H:%M:%S").unwrap_or_else(|err| {
        eprintln!("Error parsing time: {}", err);
        std::process::exit(1);
    });

    // Combine the date and time to form a NaiveDateTime (local time, no time zone info)
    let naive_datetime = NaiveDateTime::new(date, time);

    // Convert NaiveDateTime in local time to DateTime in UTC
    let local_datetime = Local.from_local_datetime(&naive_datetime).unwrap();

    println!("🗓️  {}", local_datetime);

    // Convert the local DateTime to UTC
    local_datetime.with_timezone(&Utc)
}

fn get_local_naive_date_from_utc_datetime(date: DateTime<Utc>) -> NaiveDate {
    let local_date = date.with_timezone(&Local);


    NaiveDate::from_ymd_opt(local_date.year(), local_date.month(), local_date.day())
            .unwrap_or_default()
}

fn get_local_time_as_str(date: DateTime<Utc>) -> String {
    // Convert min_date to local time
    let local_date = date.with_timezone(&Local);

    // Turn the time from local_date into a "HH:MM:SS" string
    let initial_value = format!(
        "{:02}:{:02}:{:02}",
        local_date.hour(),
        local_date.minute(),
        local_date.second()
    );

    initial_value
}
