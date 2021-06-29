use crate::live_game::GameState;
use crate::schedule::ScheduleState;
use crate::standings::StandingsState;
use mlb_api::live::LiveResponse;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MenuItem {
    Scoreboard,
    Gameday,
    Stats,
    Standings,
    Help,
    DatePicker,
}

pub struct App {
    pub active_tab: MenuItem,
    pub previous_tab: MenuItem,
    pub debug_state: DebugState,
    pub schedule: ScheduleState,
    pub date_input: DateInput,
    pub live_game: GameState,
    pub gameday: GamedayPanels,
    pub boxscore_tab: HomeOrAway,
    pub standings: StandingsState,
    pub full_screen: bool,
}

impl App {
    pub fn update_live_data(&mut self, live_data: &LiveResponse) {
        self.live_game.update(live_data);
    }
    pub fn update_tab(&mut self, next: MenuItem) {
        self.previous_tab = self.active_tab;
        self.active_tab = next;
        self.debug_state = DebugState::Off;
    }
    pub fn exit_help(&mut self) {
        if self.active_tab == MenuItem::Help {
            self.active_tab = self.previous_tab;
        }
    }
    pub fn toggle_debug(&mut self) {
        match self.debug_state {
            DebugState::Off => self.debug_state = DebugState::On,
            DebugState::On => self.debug_state = DebugState::Off,
        }
    }
    pub fn toggle_full_screen(&mut self) {
        self.full_screen = !self.full_screen;
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DebugState {
    On,
    Off,
}

/// A team must be either Home or Away.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HomeOrAway {
    Home = 0,
    Away = 1,
}

impl Default for HomeOrAway {
    fn default() -> Self {
        HomeOrAway::Home
    }
}

/// Get user input for the date and store whether it's valid.
pub struct DateInput {
    pub is_valid: bool,
    pub text: String,
}

impl Default for DateInput {
    fn default() -> Self {
        DateInput {
            is_valid: true,
            text: String::new(),
        }
    }
}

/// Store which panels should be rendered in the Gameday tab.
#[derive(Debug, Copy, Clone)]
pub struct GamedayPanels {
    pub info: bool,
    pub at_bat: bool,
    pub boxscore: bool,
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
