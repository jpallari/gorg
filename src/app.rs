use std::path::PathBuf;
use std::process::ExitCode;

use crate::cli;
use crate::cli::Cli;
use crate::config::Config;
use crate::db::DB;
use crate::git_cmd;
use crate::git_dir;
use crate::git_url;
use crate::tui;
use anyhow::Result;
use anyhow::bail;
use clap::{CommandFactory, Parser, error::ErrorKind};
use std::io::Write;
use termion::input::TermRead;

pub struct App {
    cli: Cli,
    cfg: Config,
}

impl App {
    fn handle_init(&self, args: &cli::InitArgs) -> Result<ExitCode> {
        let git_cmd = git_cmd::GitCmd::new(self.cfg.git_command.clone());

        let repo_url = git_url::from_parts(&args.remote)?;
        let project_path = git_url::to_path(&repo_url)?;
        log::debug!(
            "Git URL = {repo_url}, Git path = {}",
            project_path.join("/")
        );

        let project_full_path = self
            .cfg
            .projects_path
            .join(project_path.join(std::path::MAIN_SEPARATOR_STR));
        let git_dir = project_full_path.join(".git");

        if !git_dir.try_exists()? {
            let project_full_path_str = project_full_path.to_string_lossy();
            log::debug!("Directory {project_full_path_str} not found",);
            if args.no_clone {
                log::debug!("Git init for {project_full_path_str}");
                std::fs::create_dir_all(&project_full_path)?;
                git_cmd.init(&project_full_path)?;
            } else {
                log::debug!("Git clone for {} from {}", project_full_path_str, &repo_url);
                git_cmd.clone_repo(&repo_url, project_full_path.as_os_str())?;
            }
        }

        let remotes_str = git_cmd.remote_list(&project_full_path)?;
        if remotes_str
            .split('\n')
            .any(|remote| remote == &self.cfg.git_remote_name)
        {
            log::debug!(
                "Git set remote {}={} for {}",
                self.cfg.git_remote_name,
                repo_url,
                project_full_path.to_string_lossy(),
            );
            git_cmd.remote_set_url(
                &self.cfg.git_remote_name,
                &repo_url,
                project_full_path.as_os_str(),
            )?;
        } else {
            log::debug!(
                "Git add remote {}={} for {}",
                self.cfg.git_remote_name,
                repo_url,
                project_full_path.to_string_lossy(),
            );
            git_cmd.remote_add(
                &self.cfg.git_remote_name,
                &repo_url,
                project_full_path.as_os_str(),
            )?;
        }

        log::debug!(
            "Saving project to DB {}",
            self.cfg.index_file_path.to_string_lossy()
        );
        let mut db = DB::load(&self.cfg.index_file_path)?.unwrap_or_default();
        db.add(&project_path.join("/"))?;
        db.save(&self.cfg.index_file_path)?;

        Ok(ExitCode::SUCCESS)
    }

    fn load_db_or_fail(&self) -> Result<DB> {
        let Some(db) = DB::load(&self.cfg.index_file_path)? else {
            bail!(
                "DB not found at {}",
                self.cfg.index_file_path.to_string_lossy()
            );
        };
        Ok(db)
    }

    fn write_project_with_path<W: Write>(&self, w: &mut W, project: &str) -> Result<()> {
        write!(
            w,
            "{}{}{project}\n",
            self.cfg.projects_path.to_string_lossy(),
            std::path::MAIN_SEPARATOR,
        )?;
        Ok(())
    }

    fn handle_list(&self, args: &cli::ListArgs) -> Result<ExitCode> {
        let db = self.load_db_or_fail()?;
        let query = String::from(args.query.join(" "));
        log::debug!("List with query: {query}");

        let stdout = std::io::stdout().lock();
        let mut w = std::io::BufWriter::new(stdout);

        match (args.full_path, args.prefix_search) {
            (false, false) => {
                let matches = db.find_matches(&query);
                for project in matches {
                    write_project(&mut w, project)?;
                }
            }
            (false, true) => {
                let matches = db.find_by_prefix(&query);
                for project in matches {
                    write_project(&mut w, project)?;
                }
            }
            (true, false) => {
                let matches = db.find_matches(&query);
                for project in matches {
                    self.write_project_with_path(&mut w, project)?;
                }
            }
            (true, true) => {
                let matches = db.find_by_prefix(&query);
                for project in matches {
                    self.write_project_with_path(&mut w, project)?;
                }
            }
        }

        Ok(ExitCode::SUCCESS)
    }

    fn project_path(&self, project: &str) -> PathBuf {
        self.cfg.projects_path.join(project)
    }

    fn handle_run(&self, args: &cli::RunArgs) -> Result<ExitCode> {
        if args.command.len() == 0 {
            log::error!("No command specified");
            return Ok(ExitCode::FAILURE);
        }

        let db = self.load_db_or_fail()?;
        let query = args.query.as_deref().unwrap_or_default();

        if args.dry {
            for item in db.find_matches(&query) {
                eprintln!("dry! {item}: {}", args.command.join(" "));
            }
            Ok(ExitCode::SUCCESS)
        } else {
            let mut success = true;
            for item in db.find_matches(&query) {
                if !args.quiet {
                    eprintln!("{item}: {}", args.command.join(" "));
                }
                let dir = self.project_path(item);
                let program = &args.command[0];
                let args = &args.command[1..];
                let status = std::process::Command::new(program)
                    .args(args)
                    .current_dir(&dir)
                    .spawn()?
                    .wait()?;
                success &= status.success();
            }
            Ok(if success {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            })
        }
    }

    fn handle_find(&self, args: &cli::FindArgs) -> Result<ExitCode> {
        let mut query = String::from(args.query.join(" "));

        let db = self.load_db_or_fail()?;
        let db_view = db.view();
        let mut results = Vec::with_capacity(self.cfg.max_find_items);
        db_view.find_matches(&query, &mut results);

        let print_project = |project: &str| {
            if args.full_path {
                let path = self.cfg.projects_path.join(project);
                println!("{}", &path.to_string_lossy());
            } else {
                println!("{project}");
            }
        };

        if results.len() == 1 {
            let project = results[0].0;
            print_project(project);
            return Ok(ExitCode::SUCCESS);
        }

        let mut selection = None;
        {
            let stderr = std::io::stderr();
            let stdin = std::io::stdin();
            let mut ui = tui::PromptUI::new(stderr, &query)?;
            ui.render(
                results
                    .iter()
                    .take(self.cfg.max_find_items)
                    .map(|(item, _)| *item),
            )?;

            for event in stdin.events() {
                let ui_event = ui.handle_event(event?);
                match ui_event {
                    Some(tui::PromptUIEvent::SelectionDone) => {
                        let selected_item = ui.selected_item() as usize;
                        if selected_item < results.len() {
                            selection = Some(selected_item);
                            break;
                        }
                    }
                    Some(tui::PromptUIEvent::Exit) => break,
                    Some(tui::PromptUIEvent::PromptUpdated) => {
                        query.clear();
                        query.extend(ui.text_input());
                        db_view.find_matches(&query, &mut results);
                    }
                    Some(tui::PromptUIEvent::SelectionUpdated) => {}
                    Some(tui::PromptUIEvent::CursorUpdated) => {}
                    None => {}
                }
                if ui_event.is_some() {
                    ui.render(
                        results
                            .iter()
                            .take(self.cfg.max_find_items)
                            .map(|(item, _)| *item),
                    )?;
                }
            }
        }

        if let Some(index) = selection {
            let project = results[index].0;
            print_project(project);
        }
        Ok(ExitCode::SUCCESS)
    }

    fn handle_update_index(&self) -> Result<ExitCode> {
        if !std::fs::exists(&self.cfg.projects_path)? {
            log::error!(
                "Project directory does not exist: {}",
                &self.cfg.projects_path.to_string_lossy(),
            );
            return Ok(ExitCode::FAILURE);
        }

        let iter =
            git_dir::GitDirIterator::new(self.cfg.projects_path.clone()).filter_map(
                |res| match res {
                    Ok(dir) => match dir
                        .strip_prefix(&self.cfg.projects_path)
                        .expect("Project dir should be prefix of iterated dirs")
                        .to_str()
                    {
                        Some(dir) => Some(String::from(dir)),
                        None => {
                            log::error!(
                                "Cannot read directory as a string: {}",
                                dir.to_string_lossy()
                            );
                            None
                        }
                    },
                    Err(err) => {
                        log::error!("Failed to read file: {}", err);
                        None
                    }
                },
            );
        let db = DB::from_entries(iter);
        db.save(&self.cfg.index_file_path)?;
        Ok(ExitCode::SUCCESS)
    }

    fn handle(&mut self) -> Result<ExitCode> {
        match &self.cli.command {
            Some(cli::Commands::Init(args)) => self.handle_init(&args),
            Some(cli::Commands::List(args)) => self.handle_list(&args),
            Some(cli::Commands::Run(args)) => self.handle_run(&args),
            Some(cli::Commands::Find(args)) => self.handle_find(&args),
            Some(cli::Commands::UpdateIndex) => self.handle_update_index(),
            None => {
                let mut cmd = Cli::command();
                cmd.error(ErrorKind::MissingSubcommand, "No sub-command specified")
                    .exit();
            }
        }
    }
}

pub fn run() -> Result<ExitCode> {
    env_logger::init();
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => match err.kind() {
            ErrorKind::DisplayHelp => {
                eprintln!("{}", err);
                return Ok(ExitCode::FAILURE);
            }
            _ => return Err(err.into()),
        },
    };

    let cfg = match cli.config.as_ref() {
        Some(config_path) => Config::read_from_file(config_path)?,
        None => Config::from_env()?,
    };
    let mut app = App { cli, cfg };
    app.handle()
}

fn write_project<W: Write>(w: &mut W, project: &str) -> Result<()> {
    write!(w, "{project}\n")?;
    Ok(())
}
