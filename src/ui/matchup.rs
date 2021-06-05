use crate::matchup::Matchup;
use crate::ui::layout::LayoutAreas;

use tui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Row, StatefulWidget, Table, Widget},
};

pub struct MatchupWidget {}

impl StatefulWidget for MatchupWidget {
    type State = Matchup;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunks = LayoutAreas::for_info(area);

        Widget::render(
            Table::new(state.to_table().iter().map(|row| Row::new(row.clone())))
                .widths(&[Constraint::Length(12), Constraint::Length(25)])
                .column_spacing(1)
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::NONE)),
            chunks[0],
            buf,
        );
    }
}
