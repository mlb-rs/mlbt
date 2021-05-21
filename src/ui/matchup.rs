use crate::matchup::Matchup;
use tui::layout::{Alignment, Direction, Layout};
use tui::style::Color;
use tui::widgets::Paragraph;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Borders},
    Frame,
};

impl Matchup {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let _chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(30), // game info
                    Constraint::Percentage(70), // inning plays
                ]
                .as_ref(),
            )
            .split(rect);

        let border_style = Style::default();

        let bottom_block = Block::default()
            .borders(Borders::LEFT)
            .border_style(border_style);

        let style = Style::default().fg(Color::White);

        let help = Paragraph::new(self.to_string())
            .alignment(Alignment::Left)
            .block(bottom_block)
            .style(style);

        f.render_widget(help, rect);
    }
}
