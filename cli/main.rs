use clap::{command, Args, Parser, Subcommand};
use colored::{Color, Colorize};
use config::{Config, ConfigError};
use std::process::exit;

mod config;

#[derive(Parser)]
#[command(author, version, about = "sitt is a Simple Time Trackin application")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Start time tracking on a project")]
    Start,
    #[command(about = "Stop time tracking on a project")]
    Stop,
    #[command(subcommand, about = "Manage projects")]
    Project(ProjectCommand),
    #[command(about = "Run inital setup")]
    Setup,
}

#[derive(Subcommand)]
enum ProjectCommand {
    #[command(about = "Create a project")]
    Create(ProjectArgs),
    #[command(about = "Update a project")]
    Update,
    #[command(about = "Delete a project")]
    Delete,
    #[command(visible_alias = "ls", about = "List a projects")]
    List,
}

#[derive(Args)]
struct ProjectArgs {
    #[arg(short, long)]
    name: String,
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
            Command::Setup => {
                config::Config::setup();
            }
            Command::Start => println!("CHECKING IN..."),
            Command::Stop => println!("CHECKING OUT..."),
            Command::Project(project_command) => match project_command {
                ProjectCommand::Create(args) => println!("Creating project... {}", args.name),
                ProjectCommand::Update => println!("Update project..."),
                ProjectCommand::Delete => println!("Delete project..."),
                ProjectCommand::List => println!("List projects..."),
            },
        }
    }
}

pub fn main() {
    Command::exec();
}
