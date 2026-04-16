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
        self.state.scroll_down_by(1);
    }

    pub fn previous(&mut self) {
        self.state.scroll_up_by(1);
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
