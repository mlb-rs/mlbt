use crate::components::constants::TEAM_IDS;
use crate::state::app_settings::{AppSettings, compute_timezone_abbreviation};
use anyhow::Context;
use chrono_tz::Tz;
use chrono_tz::Tz::US__Pacific;
use directories::ProjectDirs;
use log::{LevelFilter, error};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static CONFIG_FILE_LOCATION: OnceLock<Option<PathBuf>> = OnceLock::new();
pub const DEFAULT_TIMEZONE: Tz = US__Pacific;
pub const DEFAULT_LOG_LEVEL: LogLevel = LogLevel::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Off,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// `mlbt.toml` reader/writer.
///
/// - Fields are `Option` so a partial or hand-edited file still parses.
/// - Missing fields fall back to defaults when converting into `AppSettings`.
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
            timezone: Some(DEFAULT_TIMEZONE),
            log_level: Some(DEFAULT_LOG_LEVEL),
        }
    }
}

impl From<ConfigFile> for AppSettings {
    fn from(file: ConfigFile) -> Self {
        let favorite_team = file
            .favorite_team
            .as_deref()
            .and_then(|name| TEAM_IDS.get(name).copied());
        let timezone = file.timezone.unwrap_or(DEFAULT_TIMEZONE);
        let timezone_abbreviation = compute_timezone_abbreviation(timezone);
        let log_level = file.log_level.unwrap_or(DEFAULT_LOG_LEVEL);
        Self {
            favorite_team,
            full_screen: false,
            timezone,
            timezone_abbreviation,
            log_level,
        }
    }
}

impl From<&AppSettings> for ConfigFile {
    fn from(s: &AppSettings) -> Self {
        Self {
            favorite_team: s.favorite_team.map(|t| t.name.to_string()),
            timezone: Some(s.timezone),
            log_level: Some(s.log_level),
        }
    }
}

/// Filesystem settings store. Reads and writes `mlbt.toml` in the platform config directory.
pub struct TomlFileStore {
    path: Option<PathBuf>,
}

impl Default for TomlFileStore {
    fn default() -> Self {
        Self {
            path: Self::default_path(),
        }
    }
}

impl TomlFileStore {
    const HEADER: &str = "# See https://github.com/mlb-rs/mlbt#config for options\n";
    const CONFIG_FILE_NAME: &str = "mlbt.toml";

    /// Build a store backed by a specific path. Used in tests so the real config file isn't clobbered.
    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }

    /// Resolve the canonical config path:
    /// * Linux:   /home/alice/.config/mlbt/mlbt.toml
    /// * Windows: C:\Users\Alice\AppData\Roaming\mlbt\mlbt.toml
    /// * macOS:   /Users/Alice/Library/Application Support/mlbt/mlbt.toml
    pub fn default_path() -> Option<PathBuf> {
        CONFIG_FILE_LOCATION
            .get_or_init(|| {
                if let Some(proj_dirs) = ProjectDirs::from("", "", "mlbt") {
                    let dir = proj_dirs.config_dir();
                    if !dir.exists()
                        && let Err(err) = std::fs::create_dir_all(dir)
                    {
                        error!("could not create config dir: {err:?}");
                    }
                    Some(dir.join(Self::CONFIG_FILE_NAME))
                } else {
                    error!("could not get valid home directory for config file");
                    None
                }
            })
            .clone()
    }

    pub fn load(&self) -> anyhow::Result<AppSettings> {
        let path = self.path.as_ref().context("could not find config file")?;
        if !path.exists() {
            Self::write(path, &ConfigFile::default())?;
        }
        let contents = std::fs::read_to_string(path).context("could not read config file")?;
        let file: ConfigFile =
            toml::from_str(&contents).context("could not deserialize config file")?;
        Ok(file.into())
    }

    pub fn save(&self, settings: &AppSettings) -> anyhow::Result<()> {
        let path = self.path.as_ref().context("could not find config file")?;
        Self::write(path, &ConfigFile::from(settings))
    }

    fn write(path: &Path, file: &ConfigFile) -> anyhow::Result<()> {
        let body = toml::to_string(file).context("could not serialize config")?;
        // always rewrite the default header comment
        let contents = format!("{}{body}", Self::HEADER);
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, contents).context("could not write config file")?;
        std::fs::rename(&tmp, path).context("could not finalize config file")?;
        Ok(())
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
        }
    }
}
