use crate::components::live_game::GameState;
use crate::components::schedule::ScheduleState;
use crate::components::standings::StandingsState;
use crate::components::stats::StatsState;
use crate::config::{CONFIG_LOCATION, TIMEZONE, generate_config_file, load_config_file};
use chrono::{NaiveDate, ParseError, Utc};
use chrono_tz::Tz;
use crossbeam_channel::{Receiver, Sender, bounded};
use mlb_api::client::{MLBApi, MLBApiBuilder};
use mlb_api::live::LiveResponse;
use mlb_api::schedule::ScheduleResponse;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum MenuItem {
    #[default]
    Scoreboard,
    Gameday,
    Stats,
    Standings,
    Help,
    DatePicker,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum DebugState {
    On,
    #[default]
    Off,
}

/// A team must be either Home or Away.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum HomeOrAway {
    #[default]
    Home = 0,
    Away = 1,
}

/// Get user input for the date and store whether it's valid.
pub struct DateInput {
    pub is_valid: bool,
    pub text: String,
}

/// Store which panels should be rendered in the Gameday tab.
#[derive(Debug, Copy, Clone)]
pub struct GamedayPanels {
    pub info: bool,
    pub at_bat: bool,
    pub boxscore: bool,
}

#[derive(Default)]
pub struct AppState {
    pub active_tab: MenuItem,
    pub previous_tab: MenuItem,
    pub debug_state: DebugState,
    pub schedule: ScheduleState,
    pub date_input: DateInput,
    pub live_game: GameState,
    pub gameday: GamedayPanels,
    pub boxscore_tab: HomeOrAway,
    pub standings: StandingsState,
    pub stats: StatsState,
}

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub favorite_team: Option<String>,
    pub full_screen: bool,
    pub timezone: Tz,
}

pub struct App {
    pub settings: AppSettings,
    pub state: AppState,

    pub client: MLBApi,
    pub redraw_channel: (Sender<()>, Receiver<()>),
    pub update_channel: (Sender<MenuItem>, Receiver<MenuItem>),
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            state: AppState::default(),
            settings: AppSettings {
                favorite_team: None,
                full_screen: false,
                timezone: TIMEZONE,
            },
            client: MLBApiBuilder::default().build().unwrap(),
            redraw_channel: bounded(1),
            update_channel: bounded(1),
        };
        // if config file can't be loaded just print an error message but don't block starting app
        if let Err(err) = app.load_config() {
            eprintln!("could not load config file: {:?}", err);
        }
        app
    }

    fn load_config(&mut self) -> anyhow::Result<()> {
        if let Some(path) = CONFIG_LOCATION.clone() {
            if !path.exists() {
                generate_config_file(&path)?;
            }
            let config = load_config_file(&path)?;
            self.settings.favorite_team = config.validate_favorite_team();
        } else {
            eprintln!("could not find config file");
        };
        Ok(())
    }

    pub fn update_schedule(&mut self, schedule: &ScheduleResponse) {
        self.state.schedule.update(&self.settings, schedule);
    }

    pub fn update_live_data(&mut self, live_data: &LiveResponse) {
        self.state.live_game.update(live_data);
    }

    pub fn update_tab(&mut self, next: MenuItem) {
        if self.state.active_tab != next {
            self.state.previous_tab = self.state.active_tab;
            self.state.active_tab = next;
            self.state.debug_state = DebugState::Off;
        }
    }

    pub fn try_update_date_from_input(&mut self) -> Result<(), ParseError> {
        let valid_date = self
            .state
            .date_input
            .validate_input(self.settings.timezone)?;

        // current tab is date picker, so use previous tab to update correct date
        match self.state.previous_tab {
            MenuItem::Scoreboard => self.state.schedule.set_date_from_valid_input(valid_date),
            MenuItem::Standings => self.state.standings.set_date_from_valid_input(valid_date),
            MenuItem::Stats => self.state.stats.set_date_from_valid_input(valid_date),
            _ => (),
        }
        Ok(())
    }

    pub fn move_date_selector_by_arrow(&mut self, right_arrow: bool) {
        let date = match self.state.previous_tab {
            MenuItem::Scoreboard => Some(self.state.schedule.set_date_with_arrows(right_arrow)),
            MenuItem::Standings => Some(self.state.standings.set_date_with_arrows(right_arrow)),
            MenuItem::Stats => Some(self.state.stats.set_date_with_arrows(right_arrow)),
            _ => None,
        };
        self.state.date_input.text.clear();
        if let Some(date) = date {
            self.state.date_input.text.push_str(&date.to_string());
        }
    }

    pub fn exit_help(&mut self) {
        if self.state.active_tab == MenuItem::Help {
            self.state.active_tab = self.state.previous_tab;
        }
    }

    pub fn toggle_debug(&mut self) {
        match self.state.debug_state {
            DebugState::Off => self.state.debug_state = DebugState::On,
            DebugState::On => self.state.debug_state = DebugState::Off,
        }
    }

    pub fn toggle_full_screen(&mut self) {
        self.settings.full_screen = !self.settings.full_screen;
    }
}

impl DateInput {
    pub fn validate_input(&mut self, tz: Tz) -> Result<NaiveDate, ParseError> {
        let input: String = self.text.drain(..).collect();
        let date = match input.as_str() {
            "today" => Ok(Utc::now().with_timezone(&tz).date_naive()),
            _ => NaiveDate::parse_from_str(input.as_str(), "%Y-%m-%d"),
        };
        self.is_valid = date.is_ok();
        date
    }
}

impl Default for DateInput {
    fn default() -> Self {
        DateInput {
            is_valid: true,
            text: String::new(),
        }
    }
}

impl GamedayPanels {
    /// Return the number of panels that are active.
    pub fn count(&self) -> usize {
        self.info as usize + self.at_bat as usize + self.boxscore as usize
    }
}

impl Default for GamedayPanels {
    fn default() -> Self {
        GamedayPanels {
            info: true,
            at_bat: true,
            boxscore: false,
        }
    }
}
