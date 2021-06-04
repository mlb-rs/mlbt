use crate::schedule::{Schedule, ScheduleState};
use crate::ui::layout::LayoutAreas;
use tui::widgets::{TableState, Widget};
use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, StatefulWidget, Table},
    Frame,
};

const HEADER: &[&str; 4] = &["away", "home", "time [PST]", "status"];

pub struct ScheduleWidget {}

impl StatefulWidget for ScheduleWidget {
    type State = ScheduleState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header_cells = HEADER.iter().map(|h| Cell::from(*h));
        let header = Row::new(header_cells).height(1).bottom_margin(1).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );

        let rows = state
            .schedule
            .game_info
            .iter()
            .map(|r| Row::new(r.clone()).height(1).bottom_margin(1));

        let selected_style = Style::default().bg(Color::Blue).fg(Color::Black);
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
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(25),
                Constraint::Percentage(44),
            ]);

        StatefulWidget::render(t, area, buf, &mut state.state);
    }
}
