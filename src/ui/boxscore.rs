use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::boxscore::BoxScore;

impl BoxScore {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let mut widths = vec![Constraint::Length(4); self.header.len()];
        // the first width needs to be wider to display the team name
        widths[0] = Constraint::Length(10);

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
        .widths(widths.as_slice())
        .column_spacing(1)
        .style(Style::default().fg(Color::White))
        .header(header)
        .block(Block::default().borders(Borders::ALL));

        f.render_widget(t, rect);
    }
}
