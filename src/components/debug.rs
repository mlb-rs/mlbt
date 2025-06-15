use crate::app::App;
use crate::components::game::win_probability::WinProbabilityAtBat;
use std::fmt;
use tui::Frame;

pub struct DebugInfo {
    pub game_id: Option<u64>,
    pub gameday_url: String,
    pub terminal_width: u16,
    pub terminal_height: u16,
    pub win_probability: WinProbabilityAtBat,
    pub selected_at_bat: Option<u8>,
}

impl fmt::Display for DebugInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "game id: {}, gameday: {}\nterminal height: {} width: {}\n{:?}\n{:?}",
            self.game_id.unwrap_or_default(),
            self.gameday_url,
            self.terminal_height,
            self.terminal_width,
            self.win_probability,
            self.selected_at_bat,
        )
    }
}

impl DebugInfo {
    pub fn new() -> Self {
        DebugInfo {
            game_id: None,
            gameday_url: "https://www.mlb.com/scores".to_string(),
            terminal_width: 0,
            terminal_height: 0,
            win_probability: WinProbabilityAtBat::default(),
            selected_at_bat: None,
        }
    }
    // TODO add more info
    // - last api call time
    // - other things?
    pub fn gather_info(&mut self, f: &Frame, app: &App) {
        self.game_id = app.state.schedule.get_selected_game_opt();
        self.gameday_url = format!(
            "https://www.mlb.com/gameday/{}",
            self.game_id.unwrap_or_default()
        );
        self.terminal_width = f.area().width;
        self.terminal_height = f.area().height;
        self.win_probability = app
            .state
            .gameday
            .game
            .win_probability
            .at_bats
            .get(&app.state.gameday.selected_at_bat().unwrap_or_default())
            .cloned()
            .unwrap_or_default();
        self.selected_at_bat = app.state.gameday.selected_at_bat();
    }
}
