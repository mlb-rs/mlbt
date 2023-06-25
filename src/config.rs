use crate::components::constants::TEAM_NAMES;
use anyhow::Context;
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub static CONFIG_LOCATION: Lazy<Option<PathBuf>> = Lazy::new(get_config_location);
static CONFIG_FILE: &str = "mlbt.toml";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    /// See the `TEAM_NAMES` map in `components/constants.rs` for options.
    pub favorite_team: Option<String>,
    // TODO
    // pub time_zone: Option<String>
}

impl Config {
    pub fn validate_favorite_team(&self) -> Option<String> {
        if let Some(favorite) = &self.favorite_team {
            if TEAM_NAMES.contains_key(favorite.as_str()) {
                return Some(favorite.to_string());
            }
        }
        None
    }
}

/// Generate the path of the config file for the current operating system:
/// * Linux:   /home/alice/.config/mlbt/mlbt.toml
/// * Windows: C:\Users\Alice\AppData\Roaming\mlbt\mlbt.toml
/// * macOS:   /Users/Alice/Library/Application Support/mlbt/mlbt.toml
fn get_config_location() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "mlbt") {
        let dir = proj_dirs.config_dir();
        if !dir.exists() {
            if let Err(err) = std::fs::create_dir_all(dir) {
                eprintln!("could not create config dir: {err:?}");
            }
        }
        let config_file = dir.join(CONFIG_FILE);
        Some(config_file)
    } else {
        eprintln!("could not get valid home directory for config file");
        None
    }
}

pub fn generate_config_file(path: &PathBuf) -> anyhow::Result<()> {
    let contents = toml::to_string(&Config::default()).context("could not serialize config")?;
    let contents = format!("# See https://github.com/mlb-rs/mlbt#config for options\n{contents}");
    std::fs::write(path, contents).context("could not write config file")
}

pub fn load_config_file(path: &PathBuf) -> anyhow::Result<Config> {
    let contents = std::fs::read_to_string(path).context("could not read config file")?;
    toml::from_str(&contents).context("could not deserialize config file")
}
