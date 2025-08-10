mod app;
mod cli;
mod config;
mod db;
mod fuzzy;
mod git_cmd;
mod git_dir;
mod git_url;
mod text;
mod tui;

use std::process::ExitCode;

use anyhow::Result;

fn main() -> Result<ExitCode> {
    app::run()
}
