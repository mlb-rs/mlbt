use crate::app::{App, GamedayPanels};
use std::fmt;
use tui::backend::Backend;
use tui::Frame;

pub struct DebugInfo {
    pub game_id: u64,
    pub gameday_url: String,
    pub terminal_width: u16,
    pub terminal_height: u16,
    pub gameday_active_views: GamedayPanels,
}

impl fmt::Display for DebugInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "game id: {}\ngameday: {}\nterminal height: {} width: {}\n{:?}",
            self.game_id,
            self.gameday_url,
            self.terminal_height,
            self.terminal_width,
            self.gameday_active_views
        )
    }
}

impl DebugInfo {
    pub fn new() -> Self {
        DebugInfo {
            game_id: 0,
            gameday_url: "https://www.mlb.com/scores".to_string(),
            terminal_width: 0,
            terminal_height: 0,
            gameday_active_views: GamedayPanels::default(),
        }
    }
    // TODO add more info
    // - last api call time
    // - other things?
    pub fn gather_info<B>(&mut self, f: &Frame<B>, app: &App)
    where
        B: Backend,
    {
        self.game_id = app.schedule.get_selected_game();
        self.gameday_url = format!("https://www.mlb.com/gameday/{}", self.game_id);
        self.terminal_width = f.size().width;
        self.terminal_height = f.size().height;
        self.gameday_active_views = app.gameday;
    }
}
