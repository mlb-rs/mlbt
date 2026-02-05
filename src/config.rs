use crate::components::constants::TEAM_IDS;
use crate::components::standings::Team;
use crate::state::app_settings::AppSettings;
use anyhow::Context;
use chrono::{TimeZone, Utc};
use chrono_tz::America::Los_Angeles;
use chrono_tz::{OffsetName, Tz};
use directories::ProjectDirs;
use log::{LevelFilter, error};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Off,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    /// See the `TEAM_NAMES` map in `components/constants.rs` for options.
    pub favorite_team: Option<String>,

    /// Timezone to display game start times in. Common options are:
    /// * "US/Pacific"
    /// * "US/Mountain"
    /// * "US/Central"
    /// * "US/Eastern"
    ///
    /// For the full list see https://en.wikipedia.org/wiki/List_of_tz_database_time_zones.
    pub timezone: Option<Tz>,

    /// Optional log level to use. If not present, the default is `Error`.
    /// Set the level using a lowercase string, e.g. "error".
    pub log_level: Option<LogLevel>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            favorite_team: None,
            timezone: Some(ConfigFile::DEFAULT_TIMEZONE),
            log_level: None,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<AppSettings> for ConfigFile {
    fn into(self) -> AppSettings {
        AppSettings {
            favorite_team: self.validate_favorite_team(),
            full_screen: false,
            timezone: self.validate_timezone(),
            timezone_abbreviation: self.get_timezone_abbreviation(),
            log_level: self.validate_log_level(),
        }
    }
}

static CONFIG_FILE_LOCATION: OnceLock<Option<PathBuf>> = OnceLock::new();

impl ConfigFile {
    const DEFAULT_TIMEZONE: Tz = Los_Angeles;
    const CONFIG_FILE_NAME: &'static str = "mlbt.toml";

    pub fn load_from_file() -> anyhow::Result<ConfigFile> {
        if let Some(path) = Self::get_config_location() {
            if !path.exists() {
                Self::generate_config_file(&path)?;
            }
            Self::load_config_file(&path)
        } else {
            anyhow::bail!("could not find config file");
        }
    }

    fn validate_favorite_team(&self) -> Option<Team> {
        if let Some(favorite) = &self.favorite_team
            && let Some(team) = TEAM_IDS.get(favorite.as_str())
        {
            return Some(*team);
        }
        None
    }

    fn validate_timezone(&self) -> Tz {
        self.timezone.unwrap_or(Self::DEFAULT_TIMEZONE)
    }

    fn validate_log_level(&self) -> Option<LevelFilter> {
        self.log_level.map(|level| match level {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
        })
    }

    /// Get the abbreviated name of the configured timezone, (e.g. "PST" or "PDT")
    fn get_timezone_abbreviation(&self) -> String {
        let tz = self.timezone.unwrap_or(Self::DEFAULT_TIMEZONE);
        let now = Utc::now().with_timezone(&tz).naive_utc();
        let offset = tz.offset_from_utc_datetime(&now);
        offset.abbreviation().unwrap_or("~~").to_string()
    }

    /// Generate the path of the config file for the current operating system:
    /// * Linux:   /home/alice/.config/mlbt/mlbt.toml
    /// * Windows: C:\Users\Alice\AppData\Roaming\mlbt\mlbt.toml
    /// * macOS:   /Users/Alice/Library/Application Support/mlbt/mlbt.toml
    pub fn get_config_location() -> Option<PathBuf> {
        CONFIG_FILE_LOCATION
            .get_or_init(|| {
                if let Some(proj_dirs) = ProjectDirs::from("", "", "mlbt") {
                    let dir = proj_dirs.config_dir();
                    if !dir.exists()
                        && let Err(err) = std::fs::create_dir_all(dir)
                    {
                        error!("could not create config dir: {err:?}");
                    }
                    let config_file = dir.join(Self::CONFIG_FILE_NAME);
                    Some(config_file)
                } else {
                    error!("could not get valid home directory for config file");
                    None
                }
            })
            .clone()
    }

    fn generate_config_file(path: &PathBuf) -> anyhow::Result<()> {
        let contents =
            toml::to_string(&ConfigFile::default()).context("could not serialize config")?;
        let contents =
            format!("# See https://github.com/mlb-rs/mlbt#config for options\n{contents}");
        std::fs::write(path, contents).context("could not write config file")
    }

    fn load_config_file(path: &PathBuf) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path).context("could not read config file")?;
        toml::from_str(&contents).context("could not deserialize config file")
    }
}
