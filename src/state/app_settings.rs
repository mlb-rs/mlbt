use crate::components::standings::Team;
use crate::config::{ConfigFile, LogLevel, TomlFileStore};
use chrono::{TimeZone, Utc};
use chrono_tz::{OffsetName, Tz};
use log::error;

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub favorite_team: Option<Team>,
    pub full_screen: bool,
    pub timezone: Tz,
    pub timezone_abbreviation: String,
    pub log_level: LogLevel,
}

impl AppSettings {
    /// Load settings via the given store. On load failure, logs and falls back to defaults so the
    /// app can still start.
    pub fn load(store: &TomlFileStore) -> Self {
        store.load().unwrap_or_else(|err| {
            error!("could not load settings: {err}");
            AppSettings::from(ConfigFile::default())
        })
    }

    /// Recompute the cached timezone abbreviation from the current `timezone`.
    /// Call after mutating `timezone` so the ui stays in sync.
    pub fn refresh_timezone_abbreviation(&mut self) {
        self.timezone_abbreviation = compute_timezone_abbreviation(self.timezone);
    }
}

/// Abbreviated name of a timezone at the current instant (e.g. "PST" or "PDT").
pub fn compute_timezone_abbreviation(tz: Tz) -> String {
    let now = Utc::now().with_timezone(&tz).naive_utc();
    let offset = tz.offset_from_utc_datetime(&now);
    offset.abbreviation().unwrap_or("~~").to_string()
}
