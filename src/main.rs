mod cli;
mod config;
mod db;
mod fuzzy;
mod git_dir;
mod git_ops;
mod git_url;

use std::process::ExitCode;

use crate::db::DB;
use anyhow::Result;
use clap::{CommandFactory, Parser, error::ErrorKind};
use cli::Cli;
use config::Config;
use std::io::Write;

fn main() -> Result<ExitCode> {
    env_logger::init();

    let cli = Cli::try_parse()?;
    let cfg = match cli.config {
        Some(config_path) => Config::read_from_file(config_path)?,
        None => Config::from_env()?,
    };

    match cli.command {
        Some(cli::Commands::Browse { query }) => {}
        Some(cli::Commands::Init { remote }) => {
            // git_ops::init(&cfg, remote)?;
        }
        Some(cli::Commands::Ls { query }) => {
            let db = DB::load(&cfg.db_path)?;
            let query = String::from(query.join(" "));
            let kw = fuzzy::Keywords::new(&query);

            let stdout = std::io::stdout().lock();
            let mut w = std::io::BufWriter::new(stdout);
            for m in db.find_matches(&kw) {
                write!(&mut w, "{m}\n")?;
            }
        }
        Some(cli::Commands::Run {
            query,
            dry,
            quiet,
            command,
        }) => {
            if command.len() == 0 {
                log::error!("No command specified");
                return Ok(ExitCode::FAILURE);
            }

            let db = DB::load(&cfg.db_path)?;
            let kw = query
                .as_ref()
                .map(|q| fuzzy::Keywords::new(q))
                .unwrap_or_default();

            if dry {
                for item in db.find_matches(&kw) {
                    eprintln!("dry! {item}: {}", command.join(" "));
                }
            } else {
                let mut success = true;
                for item in db.find_matches(&kw) {
                    if !quiet {
                        eprintln!("{item}: {}", command.join(" "));
                    }
                    let dir = cfg.project_path.join(item);
                    let program = &command[0];
                    let args = &command[1..];
                    let mut handle = std::process::Command::new(program)
                        .args(args)
                        .current_dir(&dir)
                        .spawn()?;
                    let status = handle.wait()?;
                    success &= status.success();
                }
                if !success {
                    return Ok(ExitCode::FAILURE);
                }
            }
        }
        Some(cli::Commands::Fetch { remote }) => {
            // git_ops::update(&cfg, remote)?;
        }
        Some(cli::Commands::Cd { query }) => {}
        Some(cli::Commands::UpdateIndex) => {
            if !std::fs::exists(&cfg.project_path)? {
                log::error!(
                    "Project directory does not exist: {}",
                    &cfg.project_path.to_string_lossy(),
                );
                return Ok(ExitCode::FAILURE);
            }

            let iter = git_dir::GitDirIterator::new(cfg.project_path.clone());
            let mut entries = Vec::new();
            for dir in iter {
                match dir {
                    Ok(dir) => match dir
                        .strip_prefix(&cfg.project_path)
                        .expect("Project dir should be prefix of iterated dirs")
                        .to_str()
                    {
                        Some(dir) => entries.push(String::from(dir)),
                        None => log::error!(
                            "Cannot read directory as a string: {}",
                            dir.to_string_lossy()
                        ),
                    },
                    Err(err) => log::error!("Failed to read file: {}", err),
                }
            }
            let db = DB::from_entries(&mut entries);
            db.save(&cfg.db_path)?;
        }
        None => {
            let mut cmd = Cli::command();
            cmd.error(ErrorKind::MissingSubcommand, "No sub-command specified")
                .exit();
        }
    }

    Ok(ExitCode::SUCCESS)
}
