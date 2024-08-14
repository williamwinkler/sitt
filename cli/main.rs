use clap::{command, Args, Parser, Subcommand};

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
    Init,
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

        match args.command {
            Command::Init => println!("Init..."),
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
