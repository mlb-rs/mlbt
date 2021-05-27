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
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Percentage(30), // game info
                    Constraint::Percentage(70), // inning plays
                ]
                .as_ref(),
            )
            .split(rect);

        let matchup = Paragraph::new(self.to_string())
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White));

        f.render_widget(matchup, chunks[0]);
    }
}
