use crate::debug::DebugInfo;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
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

        let border_style = Style::default();

        let bottom_block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        let style = Style::default().fg(Color::White);

        let help = Paragraph::new(self.to_string())
            .alignment(Alignment::Left)
            .block(bottom_block)
            .style(style);

        f.render_widget(help, chunks[1]);
    }
}
