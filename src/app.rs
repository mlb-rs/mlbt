use crate::gameday::Gameday;
use crate::live_game::GameState;
use crate::schedule::ScheduleState;

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
    pub boxscore_tab: BoxscoreTab,
    pub gameday: Gameday,
}

impl App {
    pub fn update(&mut self) {
        self.live_game.update();
        self.gameday.load_live_data(&self.live_game.live_data);
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
    pub fn get_boxscore_tab(&self) -> usize {
        match self.boxscore_tab {
            BoxscoreTab::Home => 0,
            BoxscoreTab::Away => 1,
        }
    }
}
