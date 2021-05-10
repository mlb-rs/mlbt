use crate::app::App;
use std::fmt;
use tui::backend::Backend;
use tui::Frame;

pub struct DebugInfo {
    pub game_id: u64,
    pub terminal_width: u16,
    pub terminal_height: u16,
}

impl fmt::Display for DebugInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "game id: {}\nterminal height: {} , width {}",
            self.game_id, self.terminal_height, self.terminal_width
        )
    }
}

impl DebugInfo {
    pub fn new() -> Self {
        DebugInfo {
            game_id: 0,
            terminal_width: 0,
            terminal_height: 0,
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
        self.terminal_width = f.size().width;
        self.terminal_height = f.size().height;
    }
}
