use crate::components::standings::Team;
use crate::config::ConfigFile;
use chrono_tz::Tz;

#[derive(Debug, Default, Clone)]
pub struct AppSettings {
    pub favorite_team: Option<Team>,
    pub full_screen: bool,
    pub timezone: Tz,
    pub timezone_abbreviation: String,
}

impl AppSettings {
    /// If config file can't be loaded just print an error message but don't block starting app
    pub fn load_from_file() -> Self {
        ConfigFile::load_from_file()
            .unwrap_or_else(|err| {
                eprintln!("could not load config file: {:?}", err);
                ConfigFile::default()
            })
            .into()
    }
}
