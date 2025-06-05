use crate::app::HomeOrAway;
use crate::components::boxscore::TeamBatterBoxscore;

use tui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, Widget},
};

const HEADER: [&str; 9] = ["player", "ab", "r", "h", "rbi", "bb", "so", "lob", "avg"];

pub struct TeamBatterBoxscoreWidget<'a> {
    pub active: HomeOrAway,
    pub boxscore: &'a TeamBatterBoxscore,
}

impl Widget for TeamBatterBoxscoreWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = 4;
        let mut widths = vec![Constraint::Length(width); HEADER.len()];
        // the first width needs to be wider to display the player name
        widths[0] = Constraint::Length(15);
        // the last width needs to be wider to display batting average
        widths[HEADER.len() - 1] = Constraint::Length(5);

        let header = Row::new(HEADER.iter().map(|h| Cell::from(*h)).collect::<Vec<Cell>>())
            .height(1)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        Widget::render(
            Table::new(
                self.boxscore
                    .to_table_row(self.active)
                    .iter()
                    .map(|row| Row::new(row.clone())),
                widths.as_slice(),
            )
            .column_spacing(0)
            .style(Style::default().fg(Color::White))
            .header(header)
            .block(Block::default().borders(Borders::NONE)),
            area,
            buf,
        );
    }
}
