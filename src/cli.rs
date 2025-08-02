use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Browse {
        query: Vec<String>,
    },
    Cd {
        query: Vec<String>,
    },
    Fetch {
        remote: Vec<String>,
    },
    Init {
        remote: Vec<String>,
    },
    Ls {
        query: Vec<String>,
    },
    Run {
        #[arg(short, long, value_name = "QUERY")]
        query: Option<String>,

        #[arg(short, long)]
        dry: bool,

        #[arg(long)]
        quiet: bool,

        command: Vec<String>,
    },
    UpdateIndex,
}

