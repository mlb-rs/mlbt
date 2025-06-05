use crate::components::linescore::LineScore;
use crate::state::app_state::HomeOrAway;

use tui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, Widget},
};

// TODO depending on the terminal size the number of columns display should be changed. Only two columns *need* to be shown, the current inning and the run totals - eveything else can get chopped off.

pub struct LineScoreWidget<'a> {
    pub active: HomeOrAway,
    pub linescore: &'a LineScore,
}

impl Widget for LineScoreWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO set dynamically based on rect size?
        let width = match self.linescore.mini {
            true => 3,
            false => 5,
        };
        let mut widths = vec![Constraint::Length(width); self.linescore.header.len()];
        // the first width needs to be wider to display the team abbreviation
        widths[0] = Constraint::Length(6);
        // extra padding before R H E
        widths[self.linescore.header.len() - 4] = Constraint::Length(width + 1);

        let header = Row::new(self.linescore.header.clone())
            .height(1)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        let t = Table::new(
            vec![
                Row::new(self.linescore.away.create_score_vec(self.active)),
                Row::new(self.linescore.home.create_score_vec(self.active)),
            ],
            widths.as_slice(),
        )
        .column_spacing(0)
        .style(Style::default().fg(Color::White))
        .header(header)
        .block(Block::default().borders(Borders::NONE));

        Widget::render(t, area, buf);
    }
}
