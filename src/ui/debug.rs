use crate::debug::DebugInfo;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

impl DebugInfo {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
            .split(rect);

        let help = Paragraph::new(self.to_string())
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default()),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(Clear, chunks[1]); //this clears out the background
        f.render_widget(help, chunks[1]);
    }
}
