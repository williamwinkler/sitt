use clap::{command, Args, Parser, Subcommand};
use colored::{Color, Colorize};
use config::{Config, ConfigError};
use project::{get_project_name_from_input, ProjectSelectOption};
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
    #[command(subcommand, about = "Manage your configuration")]
    Config(ConfigCommand),
}

#[derive(Subcommand)]
enum ProjectCommand {
    #[command(about = "Create a project")]
    Create(ProjectArgs),
    #[command(about = "Update a project")]
    Update(ProjectArgs),
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
struct ProjectArgs {
    #[arg(short, long)]
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
            Command::Start(args) => {
                let project_name = if let Some(project_name) = args.name {
                    project_name
                } else {
                    project::select_project(
                        &config,
                        "start tracking on",
                        ProjectSelectOption::InActive,
                    )
                };

                timetrack::start_time_tracking(&config, &project_name);
            }
            Command::Stop(args) => {
                let project_name = if let Some(project_name) = args.name {
                    project_name
                } else {
                    project::select_project(
                        &config,
                        "stop tracking on",
                        ProjectSelectOption::Active,
                    )
                };

                timetrack::stop_time_tracking(&config, &project_name);
            }
            Command::Project(project_command) => match project_command {
                ProjectCommand::Create(args) => {
                    let project_name = if let Some(project_name) = args.name {
                        project_name
                    } else {
                        get_project_name_from_input()
                    };
                    project::create_project(&config, project_name)
                }
                ProjectCommand::Update(args) => {
                    let project_name = if let Some(project_name) = args.name {
                        project_name
                    } else {
                        project::select_project(&config, "update", ProjectSelectOption::None)
                    };

                    project::update_project(&config, &project_name)
                }
                ProjectCommand::Delete(args) => {
                    let project_name = if let Some(project_name) = args.name {
                        project_name
                    } else {
                        project::select_project(&config, "delete", ProjectSelectOption::None)
                    };

                    project::delete_project(&config, &project_name)
                }

                ProjectCommand::Get(args) => {
                    let project_name = if let Some(project_name) = args.name {
                        project_name
                    } else {
                        project::select_project(&config, "get", ProjectSelectOption::None)
                    };

                    project::get_project_by_name(&config, &project_name)
                }
                ProjectCommand::List => project::get_projects(&config),
            },
            Command::Config(config_command) => match config_command {
                ConfigCommand::Set => {
                    Config::setup();
                }
                ConfigCommand::Get => {
                    println!("🔑 Your configuration:");
                    println!("{} URL: {}", "sitt".color(Color::Yellow), &config.get_url(),);
                    println!("API key:  {}", &config.get_api_key())
                }
            },
        }
    }
}

pub fn main() {
    Command::exec();
}
