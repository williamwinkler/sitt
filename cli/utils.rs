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

pub fn prompt_user_for_datetime(msg: &str, min_date: Option<DateTime<Utc>>) -> DateTime<Utc> {
    // Ask the user to select a date (in local time zone)
    let mut date = DateSelect::new(msg);

    if let Some(min_date) = min_date {
        // Convert min_date to local time
        let min_date_local = min_date.with_timezone(&Local);

        let naive_min_date = NaiveDate::from_ymd_opt(
            min_date_local.year(),
            min_date_local.month(),
            min_date_local.day(),
        );

        let naive_min_date = naive_min_date.unwrap_or_default();

        date = date.with_min_date(naive_min_date);
        date = date.with_starting_date(naive_min_date);
    }

    let date = date.prompt().unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        exit(1);
    });

    // Ask for time (HH:MM:SS) in local time zone
    let mut time_input = Text::new("Enter the time (HH:MM:SS in 24-hour format):")
        .with_help_message("Example: 14:30:15");

    let mut initial_value = String::new();

    if let Some(min_date) = min_date {
        // Convert min_date to local time
        let min_date_local = min_date.with_timezone(&Local);

        // Assign a formatted string to `initial_value`
        initial_value = format!(
            "{}:{}:{}",
            min_date_local.hour(),
            min_date_local.minute(),
            min_date_local.second()
        );

        // Use the variable
        time_input = time_input.with_initial_value(&initial_value);
    }

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
