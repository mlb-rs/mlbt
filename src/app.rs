use crate::schedule::StatefulSchedule;
use crate::ui::layout::LayoutAreas;
use mlb_api::MLBApi;

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

pub struct App<'a, 'b> {
    pub layout: LayoutAreas,
    pub tabs: Vec<&'a str>,
    pub previous_state: MenuItem,
    pub active_tab: MenuItem,
    pub debug_state: DebugState,
    pub schedule: &'b mut StatefulSchedule,
    pub api: &'a MLBApi,
}

impl App<'_, '_> {
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
    pub fn update_schedule(&mut self) {
        let schedule = self.api.get_todays_schedule();
        self.schedule.update(&schedule);
    }
    pub fn toggle_debug(&mut self) {
        match self.debug_state {
            DebugState::Off => self.debug_state = DebugState::On,
            DebugState::On => self.debug_state = DebugState::Off,
        }
    }
}
