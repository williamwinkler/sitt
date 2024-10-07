use clap::{command, Args, Parser, Subcommand};
use colored::{Color, Colorize};
use config::{Config, ConfigError};
use std::process::exit;

mod config;
mod project;
mod sitt_client;
mod timetrack;
mod utils;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Use this CLI tool to interact with the (Si)mple (T)ime (T)racking API ⏱️"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Start time tracking on a project")]
    Start(ProjectArgs),
    #[command(about = "Stop time tracking on a project")]
    Stop(ProjectArgs),
    #[command(subcommand, about = "Manage your projects")]
    Project(ProjectCommand),
    #[command(subcommand, about = "Manage time on your projects")]
    Time(TimeTrackCommand),
    #[command(subcommand, about = "Manage your configuration")]
    Config(ConfigCommand),
}

#[derive(Subcommand)]
enum TimeTrackCommand {
    #[command(about = "Add time on a project")]
    Add(ProjectArgs),
    #[command(about = "Delete time logged on a project")]
    Delete(ProjectArgs),
    #[command(about = "Edit a time log on a project")]
    Edit(ProjectArgs),
    #[command(visible_alias = "ls", about = "List time logged on a project")]
    List(ProjectArgs),
}

#[derive(Subcommand)]
enum ProjectCommand {
    #[command(about = "Create a project")]
    Create(ProjectArgs),
    #[command(about = "Edit the name of a project")]
    Edit(ProjectArgs),
    #[command(about = "Delete a project")]
    Delete(ProjectArgs),
    #[command(about = "Get a project by name")]
    Get(ProjectArgs),
    #[command(visible_alias = "ls", about = "List a projects")]
    List,
}

#[derive(Subcommand)]
enum ConfigCommand {
    #[command(about = "Run configuration setup")]
    Set,
    #[command(about = "Get your configuration")]
    Get,
}

#[derive(Args)]
pub struct ProjectArgs {
    #[arg(short, long, help = "Specify the name of the project")]
    name: Option<String>,
}

impl Command {
    fn exec() {
        let args = Cli::parse();

        // Ensure the configuration file is valid
        let config: Config = config::Config::load().unwrap_or_else(|err| {
            match err {
                // Assume if there is no configuration file, it's their first time
                ConfigError::MissingFile(_) => {
                    println!("👋 Hello!");
                    println!(
                        "It looks like it's your first time using {} ✨",
                        "sitt".color(Color::Yellow)
                    );
                    println!("We need to set up a few things, and then you will be ready to track time on your favorite projects! ⏱️\n");
                    config::Config::setup()
                },
                _ => {
                    eprintln!("{err}");
                    exit(1)
                },
            }
        });

        match args.command {
            Command::Start(args) => timetrack::start_time_tracking(&config, &args),
            Command::Stop(args) => timetrack::stop_time_tracking(&config, &args),
            Command::Project(project_command) => match project_command {
                ProjectCommand::Create(args) => project::create_project(&config, args),
                ProjectCommand::Edit(args) => project::update_project(&config, &args),
                ProjectCommand::Delete(args) => project::delete_project(&config, &args),
                ProjectCommand::Get(args) => project::get_project_by_name(&config, &args),
                ProjectCommand::List => project::get_projects(&config),
            },
            Command::Time(timetrack_command) => match timetrack_command {
                TimeTrackCommand::Add(args) => timetrack::add_time_tracking(&config, &args),
                TimeTrackCommand::List(args) => timetrack::get_time_trackings(&config, &args),
                TimeTrackCommand::Edit(args) => timetrack::edit_time_track(&config, &args),
                TimeTrackCommand::Delete(args) => timetrack::delete_time_tracking(&config, &args),
            },
            Command::Config(config_command) => match config_command {
                ConfigCommand::Set => {
                    Config::setup();
                }
                ConfigCommand::Get => {
                    println!("🔑 Your configuration:\n");
                    println!("{} URL: {}", "sitt".color(Color::Yellow), &config.get_url(),);
                    println!("API key:  {}", &config.get_api_key());
                }
            },
        }
    }
}

pub fn main() {
    Command::exec();
}
