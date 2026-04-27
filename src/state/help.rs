use crate::components::help::DOCS_LEN;
use tui::widgets::TableState;

pub struct HelpState {
    pub state: TableState,
}

impl Default for HelpState {
    fn default() -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        Self { state }
    }
}

impl HelpState {
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) if i >= DOCS_LEN - 1 => 0,
            Some(i) => i + 1,
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(0) | None => DOCS_LEN - 1,
            Some(i) => i - 1,
        };
        self.state.select(Some(i));
    }

    pub fn page_down(&mut self) {
        self.state.scroll_down_by(10);
    }

    pub fn page_up(&mut self) {
        self.state.scroll_up_by(10);
    }

    pub fn reset(&mut self) {
        self.state.select(Some(0));
    }
}
