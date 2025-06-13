use crate::app::App;
use std::fmt;
use tui::Frame;

pub struct DebugInfo {
    pub game_id: Option<u64>,
    pub gameday_url: String,
    pub terminal_width: u16,
    pub terminal_height: u16,
}

impl fmt::Display for DebugInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "game id: {}, gameday: {}\nterminal height: {} width: {}\n",
            self.game_id.unwrap_or_default(),
            self.gameday_url,
            self.terminal_height,
            self.terminal_width,
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
    }
}
