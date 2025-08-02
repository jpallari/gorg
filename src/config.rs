use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

const CONFIG_ENV_VAR_NAME: &str = "GORG_CONFIG";
const DEFAULT_CONFIG_DIRNAME: &str = "gorg";
const DEFAULT_CONFIG_FILENAME: &str = "config.toml";
const DEFAULT_PROJECT_DIR_NAME: &str = "Projects";
const DEFAULT_DB_FILE_NAME: &str = ".gorg-db";

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_project_path")]
    pub project_path: PathBuf,

    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,
}

fn home_dir() -> PathBuf {
    std::env::home_dir().expect("Home dir should be defined for the user")
}

fn default_project_path() -> PathBuf {
    let mut path = home_dir();
    path.push(DEFAULT_PROJECT_DIR_NAME);
    path
}

fn default_db_path() -> PathBuf {
    let mut path = default_project_path();
    path.push(DEFAULT_DB_FILE_NAME);
    path
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
            project_path: default_project_path(),
            db_path: default_db_path(),
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
