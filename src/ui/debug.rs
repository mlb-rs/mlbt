use crate::components::debug::DebugInfo;
use tui::layout::Constraint::Percentage;
use tui::prelude::*;
use tui::widgets::{Block, Borders, Clear, Paragraph};

impl DebugInfo {
    pub fn render(&self, f: &mut Frame, rect: Rect) {
        let [_, debug] = Layout::vertical([Percentage(80), Percentage(20)]).areas(rect);

        let help = Paragraph::new(self.to_string())
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default()),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(Clear, debug); //this clears out the background
        f.render_widget(help, debug);
    }
}
