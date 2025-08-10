use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to the gorg configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Command to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Find a project using a fuzzy matcher (interactive)
    Find(FindArgs),

    /// Initializes a repository for the given remote
    Init(InitArgs),

    /// List all projects that match the given fuzzy query (alias "ls")
    #[command(alias = "ls")]
    List(ListArgs),

    /// Run a given command in all (matching) projects
    Run(RunArgs),

    /// Scan the project directory for all Git projects and update the index file
    UpdateIndex,
}

#[derive(Args)]
pub struct InitArgs {
    /// Git remote
    pub remote: Vec<String>,

    /// When set, repository cloning is not performed.
    #[arg(long)]
    pub no_clone: bool,
}

#[derive(Args)]
pub struct FindArgs {
    /// Initial fuzzy find query
    pub query: Vec<String>,

    /// Print full path instead of just the project name
    #[arg(short, long)]
    pub full_path: bool,
}

#[derive(Args)]
pub struct ListArgs {
    // Fuzzy find query. All projects will be listed when not used.
    pub query: Vec<String>,

    /// Print full path instead of just the project name
    #[arg(short, long)]
    pub full_path: bool,

    // Use a prefix query instead of a fuzzy query
    #[arg(short, long)]
    pub prefix_search: bool,
}

#[derive(Args)]
pub struct RunArgs {
    /// Fuzzy find query used for selecting which projects to run the query on.
    /// When not set, all projects will be targeted.
    #[arg(short, long, value_name = "QUERY")]
    pub query: Option<String>,

    /// When enabled, only print the project names where the command would be run on.
    #[arg(short, long)]
    pub dry: bool,

    /// When enabled, project name is not printed when the command is run.
    #[arg(long)]
    pub quiet: bool,

    /// The command to run and the parameters to give to the command
    pub command: Vec<String>,
}
