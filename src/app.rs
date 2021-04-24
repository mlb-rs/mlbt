use crate::schedule::StatefulSchedule;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MenuItem {
    Scoreboard,
    GameDay,
    Stats,
    Standings,
    Help,
}

pub struct App<'a, 'b> {
    pub tabs: Vec<&'a str>,
    pub previous_state: MenuItem,
    pub active_tab: MenuItem,
    pub schedule: &'b mut StatefulSchedule,
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
}
