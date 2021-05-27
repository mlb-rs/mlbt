use crate::schedule::StatefulSchedule;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
    Frame,
};

impl StatefulSchedule {
    pub fn render<B>(&mut self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(Color::White);
        let header_cells = ["away", "home", "time [PST]", "status"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));

        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);

        let rows = self
            .schedule
            .game_info
            .iter()
            .map(|r| Row::new(r.clone()).height(1).bottom_margin(1));

        let t = Table::new(rows)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("scoreboard"),
            )
            .highlight_style(selected_style)
            .highlight_symbol(">> ")
            .widths(&[
                // TODO review these on different width terminals
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(25),
                Constraint::Percentage(45),
            ]);

        f.render_stateful_widget(t, rect, &mut self.state);
    }
}
