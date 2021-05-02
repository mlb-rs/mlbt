use crate::schedule::{create_table, get_game_pks, StatefulSchedule};
use crate::ui::layout::LayoutAreas;
use mlb_api::MLBApi;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MenuItem {
    Scoreboard,
    GameDay,
    Stats,
    Standings,
    Help,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DebugInfo {
    Test,
    None,
}

pub struct App<'a, 'b> {
    pub layout: LayoutAreas,
    pub tabs: Vec<&'a str>,
    pub previous_state: MenuItem,
    pub active_tab: MenuItem,
    pub debug_state: DebugInfo,
    pub schedule: &'b mut StatefulSchedule,
    pub api: &'a MLBApi,
}

impl App<'_, '_> {
    pub fn update_tab(&mut self, next: MenuItem) {
        self.previous_state = self.active_tab;
        self.active_tab = next;
    }
    pub fn exit_help(&mut self) {
        if self.active_tab == MenuItem::Help {
            self.active_tab = self.previous_state;
        }
    }
    pub fn update_schedule(&mut self) {
        let schedule = self.api.get_todays_schedule();
        self.schedule.items = create_table(&schedule);
        self.schedule.game_ids = get_game_pks(&schedule);
    }
}
