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
}

pub struct App {
    pub active_tab: MenuItem,
    pub previous_state: MenuItem,
    pub debug_state: DebugState,
    pub schedule: ScheduleState,
    pub live_game: GameState,
    pub gameday: GamedayPanels,
    pub boxscore_tab: BoxscoreTab,
    pub standings: StandingsState,
}

impl App {
    pub fn update_live_data(&mut self, live_data: &LiveResponse) {
        self.live_game.update(live_data);
    }
    pub fn update_tab(&mut self, next: MenuItem) {
        self.previous_state = self.active_tab;
        self.active_tab = next;
        self.debug_state = DebugState::Off;
    }
    pub fn exit_help(&mut self) {
        if self.active_tab == MenuItem::Help {
            self.active_tab = self.previous_state;
        }
    }
    pub fn toggle_debug(&mut self) {
        match self.debug_state {
            DebugState::Off => self.debug_state = DebugState::On,
            DebugState::On => self.debug_state = DebugState::Off,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DebugState {
    On,
    Off,
}

/// Store which team should be displayed in the boxscore in the Gameday tab.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BoxscoreTab {
    Home = 0,
    Away = 1,
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
    /// All panels should be active to start.
    fn default() -> Self {
        GamedayPanels {
            info: true,
            at_bat: true,
            boxscore: true,
        }
    }
}
