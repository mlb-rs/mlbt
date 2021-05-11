use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::boxscore::BoxScore;

// The longest game in MLB history was 26 innings. There are four extra columns:
// team name, runs, hits, and errors, so having a max width of 30 for the boxscore
// seems pretty safe.
const BOXSCORE_WIDTHS: &[Constraint] = &[Constraint::Length(4); 30];

impl BoxScore {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        // slice off the correct number of widths TODO is there a better way to do this?
        let widths: &[Constraint] = &BOXSCORE_WIDTHS[0..self.header.len()];
        // let widths: &[Constraint] = vec![Constraint::Length(4); header_row.len()].as_slice();

        let header = Row::new(self.header.clone())
            .height(1)
            .bottom_margin(1)
            .style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            );

        let t = Table::new(vec![
            Row::new(self.away.create_score_vec()).bottom_margin(1),
            Row::new(self.home.create_score_vec()),
        ])
        .widths(widths)
        .column_spacing(1)
        .style(Style::default().fg(Color::White))
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("box score"));

        f.render_widget(t, rect);
    }
}
