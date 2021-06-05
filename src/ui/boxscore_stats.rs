use crate::app::BoxscoreTab;
use crate::boxscore_stats::TeamBatterBoxscore;

use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, Tabs, Widget},
};

const HEADER: [&str; 9] = ["player", "ab", "r", "h", "rbi", "bb", "so", "lob", "avg"];
const HOME_AWAY: [&str; 2] = ["home", "away"];

pub struct TeamBatterBoxscoreWidget {
    pub active: BoxscoreTab,
}

impl StatefulWidget for TeamBatterBoxscoreWidget {
    type State = TeamBatterBoxscore;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunk = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(2)
            .vertical_margin(0)
            .constraints(
                [
                    Constraint::Length(8),       // score board
                    Constraint::Length(3),       // team tabs
                    Constraint::Percentage(100), // box score
                ]
                .as_ref(),
            )
            .split(area);

        Widget::render(
            Tabs::new(
                HOME_AWAY
                    .iter()
                    .map(|t| {
                        let (first, rest) = t.split_at(1);
                        Spans::from(vec![
                            Span::styled(
                                first,
                                Style::default()
                                    .add_modifier(Modifier::BOLD)
                                    .fg(Color::White),
                            ),
                            Span::styled(rest, Style::default().fg(Color::White)),
                        ])
                    })
                    .collect(),
            )
            .block(Block::default().borders(Borders::NONE))
            .select(self.active as usize)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Blue),
            ),
            chunk[1],
            buf,
        );

        let width = 3;
        let mut widths = vec![Constraint::Length(width); HEADER.len()];
        // the first width needs to be wider to display the team name
        widths[0] = Constraint::Length(15);
        // the last width needs to be wider to display batting average
        widths[HEADER.len() - 1] = Constraint::Length(5);

        let header = Row::new(HEADER.iter().map(|h| Cell::from(*h)).collect::<Vec<Cell>>())
            .height(1)
            .bottom_margin(1)
            .style(Style::default().add_modifier(Modifier::BOLD));

        Widget::render(
            Table::new(
                state
                    .to_table_row(self.active)
                    .iter()
                    .map(|row| Row::new(row.clone())),
            )
            .widths(widths.as_slice())
            .column_spacing(1)
            .style(Style::default().fg(Color::White))
            .header(header)
            .block(Block::default().borders(Borders::NONE)),
            chunk[2],
            buf,
        );
    }
}
