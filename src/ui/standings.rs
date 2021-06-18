use crate::standings::StandingsState;

use tui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, StatefulWidget, Table},
};

const HEADER: &[&str; 7] = &["Team", "W", "L", "PCT", "GB", "WCGB", "STRK"];

pub struct StandingsWidget {}

impl StatefulWidget for StandingsWidget {
    type State = StandingsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header_cells = HEADER.iter().map(|h| Cell::from(*h));
        let header = Row::new(header_cells)
            .height(1)
            .bottom_margin(1)
            .style(Style::default().add_modifier(Modifier::BOLD));

        let rows = state
            .standings
            .iter()
            .map(|s| Row::new(s.to_cells()).height(1).bottom_margin(1));

        let selected_style = Style::default().bg(Color::Blue).fg(Color::Black);
        let t = Table::new(rows)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(selected_style)
            .highlight_symbol(">> ")
            .widths(&[
                Constraint::Length(20),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
            ]);

        StatefulWidget::render(t, area, buf, &mut state.state);
    }
}
