use std::{fmt::Display, process::exit, time::Duration};

use chrono::{DateTime, Local, NaiveDateTime, NaiveTime, TimeZone, Utc};
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{DateSelect, Text};

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

pub fn prompt_user_for_datetime(msg: &str) -> DateTime<Utc> {
    // Ask the user to select a date (in local time zone)
    let date = DateSelect::new(msg)
        .prompt()
        .expect("Failed prompting date");

    // Ask for time (HH:MM:SS) in local time zone
    let time_input = Text::new("Enter the time (HH:MM:SS in 24-hour format):")
        .with_help_message("Example: 14:30:15")
        .prompt()
        .expect("Failed to get time");

    // Parse the time (in local time zone) and handle errors
    let time = NaiveTime::parse_from_str(&time_input, "%H:%M:%S").unwrap_or_else(|err| {
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
