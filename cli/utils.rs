use std::{fmt::Display, process::exit, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};

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

    return spinner;
}
