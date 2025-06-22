use crate::components::debug::DebugInfo;
use crate::ui::logs::LogWidget;
use tui::layout::Constraint::Percentage;
use tui::prelude::*;
use tui::widgets::{Block, Clear, Paragraph};

impl DebugInfo {
    pub fn render(&self, f: &mut Frame, rect: Rect, show_logs: bool) {
        let [_, area] = Layout::vertical([Percentage(80), Percentage(20)]).areas(rect);

        let debug = Paragraph::new(self.to_string())
            .alignment(Alignment::Left)
            .block(Block::bordered().title("debug"))
            .style(Style::default().fg(Color::White));

        f.render_widget(Clear, area); //this clears out the background
        if show_logs {
            f.render_widget(LogWidget {}, area);
        } else {
            f.render_widget(debug, area);
        }
    }
}
