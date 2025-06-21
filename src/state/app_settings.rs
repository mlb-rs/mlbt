use crate::components::standings::Team;
use crate::config::ConfigFile;
use chrono_tz::Tz;
use log::{LevelFilter, error};

#[derive(Debug, Default, Clone)]
pub struct AppSettings {
    pub favorite_team: Option<Team>,
    pub full_screen: bool,
    pub timezone: Tz,
    pub timezone_abbreviation: String,
    pub log_level: Option<LevelFilter>,
}

impl AppSettings {
    /// If config file can't be loaded just print an error message but don't block starting app
    pub fn load_from_file() -> Self {
        ConfigFile::load_from_file()
            .unwrap_or_else(|err| {
                error!("could not load config file: {err}");
                ConfigFile::default()
            })
            .into()
    }
}
