use crate::gameday::Gameday;
use crate::live_game::GameState;
use crate::schedule::ScheduleState;
use mlb_api::live::LiveResponse;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MenuItem {
    Scoreboard,
    Gameday,
    Stats,
    Standings,
    Help,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DebugState {
    On,
    Off,
}

pub enum BoxscoreTab {
    Home,
    Away,
}

pub struct App {
    pub active_tab: MenuItem,
    pub previous_state: MenuItem,
    pub debug_state: DebugState,
    pub schedule: ScheduleState,
    pub live_game: GameState,
    pub gameday: Gameday,
}

impl App {
    pub fn update_live_data(&mut self, live_data: &LiveResponse) {
        self.live_game.update(live_data);
        self.gameday.load_live_data(live_data);
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
