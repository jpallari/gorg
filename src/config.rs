use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

const CONFIG_ENV_VAR_NAME: &str = "GORG_CONFIG";
const DEFAULT_CONFIG_DIRNAME: &str = "gorg";
const DEFAULT_CONFIG_FILENAME: &str = "config.toml";
const DEFAULT_PROJECT_DIR_NAME: &str = "projects";
const DEFAULT_DB_FILE_NAME: &str = ".gorg-db";

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_projects_path")]
    pub projects_path: PathBuf,

    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,

    #[serde(default = "default_max_find_items")]
    pub max_find_items: usize,

    #[serde(default = "default_git_command")]
    pub git_command: String,

    #[serde(default = "default_git_remote_name")]
    pub git_remote_name: String,
}

fn home_dir() -> PathBuf {
    std::env::home_dir().expect("Home dir must be defined for the user")
}

fn default_projects_path() -> PathBuf {
    let mut path = home_dir();
    path.push(DEFAULT_PROJECT_DIR_NAME);
    path
}

fn default_db_path() -> PathBuf {
    let mut path = default_projects_path();
    path.push(DEFAULT_DB_FILE_NAME);
    path
}

fn default_max_find_items() -> usize {
    10
}

fn default_git_command() -> String {
    String::from("git")
}

fn default_git_remote_name() -> String {
    String::from("origin")
}

fn config_path() -> PathBuf {
    if let Ok(config_path) = std::env::var(CONFIG_ENV_VAR_NAME) {
        return config_path.into();
    }
    let mut path = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .ok()
        .unwrap_or_else(|| {
            let mut path = home_dir();
            path.push(".config");
            path
        });
    path.push(DEFAULT_CONFIG_DIRNAME);
    path.push(DEFAULT_CONFIG_FILENAME);
    path
}

impl Default for Config {
    fn default() -> Self {
        Config {
            projects_path: default_projects_path(),
            db_path: default_db_path(),
            max_find_items: default_max_find_items(),
            git_command: default_git_command(),
            git_remote_name: default_git_remote_name(),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Config> {
        let path = config_path();
        let path_str = path.to_string_lossy();

        log::debug!("Reading config from path: {path_str}");

        match std::fs::read_to_string(&path) {
            Ok(contents) => Self::from_str(&contents),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    log::debug!("Config not found from {path_str}. Using default configuration.");
                    Ok(Self::default())
                }
                _ => Err(e.into()),
            },
        }
    }

    pub fn read_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Config> {
        let contents = std::fs::read_to_string(&path)?;
        Self::from_str(&contents)
    }

    fn from_str(s: &str) -> Result<Config> {
        let config: Self = toml::from_str(s)?;
        Ok(config)
    }
}
