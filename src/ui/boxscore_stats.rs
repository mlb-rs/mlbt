use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::boxscore_stats::TeamBatterBoxscore;

impl TeamBatterBoxscore {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let chunk = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(2)
            .vertical_margin(1)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(rect);

        let header = vec!["player", "ab", "r", "h", "rbi", "bb", "so", "lob", "avg"];

        let width = 3;
        let mut widths = vec![Constraint::Length(width); header.len()];
        // the first width needs to be wider to display the team name
        widths[0] = Constraint::Length(15);
        widths[header.len() - 1] = Constraint::Length(5);

        let header = Row::new(header).height(1).bottom_margin(1).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );

        let t = Table::new(self.to_table_row().iter().map(|row| Row::new(row.clone())))
            .widths(widths.as_slice())
            .column_spacing(1)
            .style(Style::default().fg(Color::White))
            .header(header)
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(t, chunk[1]);
    }
}
