use crate::app::HomeOrAway;
use crate::components::linescore::LineScore;

use tui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, StatefulWidget, Table, TableState},
};

// TODO depending on the terminal size the number of columns display should be changed. Only two columns *need* to be shown, the current inning and the run totals - eveything else can get chopped off.

pub struct LineScoreWidget {
    pub active: HomeOrAway,
}

impl StatefulWidget for LineScoreWidget {
    type State = LineScore;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // TODO set dynamically based on rect size?
        let width = match state.mini {
            true => 3,
            false => 5,
        };
        let mut widths = vec![Constraint::Length(width); state.header.len()];
        // the first width needs to be wider to display the team abbreviation
        widths[0] = Constraint::Length(6);
        // extra padding before R H E
        widths[state.header.len() - 4] = Constraint::Length(width + 1);

        let header = Row::new(state.header.clone())
            .height(1)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        let t = Table::new(vec![
            Row::new(state.away.create_score_vec(self.active)),
            Row::new(state.home.create_score_vec(self.active)),
        ])
        .widths(widths.as_slice())
        .column_spacing(0)
        .style(Style::default().fg(Color::White))
        .header(header)
        .block(Block::default().borders(Borders::NONE));

        let mut table_state = TableState::default();
        StatefulWidget::render(t, area, buf, &mut table_state);
    }
}
