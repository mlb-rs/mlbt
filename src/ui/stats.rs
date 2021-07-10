use crate::stats::StatsState;

use tui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table},
};

pub struct StatsWidget {}

impl StatefulWidget for StatsWidget {
    type State = StatsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let (header, rows) = state.generate_table();

        let header = Row::new(header)
            .height(1)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        // TODO see if possible to remove another iter and clone here
        let rows: Vec<Row> = rows.iter().map(|r| Row::new(r.iter().cloned())).collect();

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
            ]);

        StatefulWidget::render(t, area, buf, &mut state.state);
    }
}
