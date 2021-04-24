#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MenuItem {
    Scoreboard,
    GameDay,
    Stats,
    Standings,
    Help,
}

pub struct App<'a> {
    pub tabs: Vec<&'a str>,
    pub previous_state: MenuItem,
    pub active_tab: MenuItem,
}

impl App<'_> {
    pub fn update_tab(&mut self, next: MenuItem) {
        self.previous_state = self.active_tab;
        self.active_tab = next;
    }
    // TODO not working
    pub fn exit_help(&mut self) {
        if self.active_tab == MenuItem::Help {
            self.active_tab = self.previous_state;
        }
    }
}
