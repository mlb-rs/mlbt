use crate::components::probable_pitchers::ProbablePitcherMatchup;
use crate::components::schedule::ScheduleRow;
use crate::ui::color::{border_style, header_style};
use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Padding, Row, Table};

const HEADER: [&str; 8] = ["", "", "W", "L", "ERA", "IP", "K", "BB"];

pub struct ProbablePitchersWidget<'a> {
    pub matchup: ProbablePitcherMatchup<'a>,
}

impl Widget for ProbablePitchersWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let use_abbreviation = area.width < ScheduleRow::ABBREVIATION_WIDTH;
        let col_width = if use_abbreviation {
            ScheduleRow::ABBREVIATION_COL_WIDTH
        } else {
            ScheduleRow::NORMAL_COL_WIDTH
        };

        let away_team = if use_abbreviation {
            self.matchup.away_team.abbreviation
        } else {
            self.matchup.away_team.team_name
        };
        let home_team = if use_abbreviation {
            self.matchup.home_team.abbreviation
        } else {
            self.matchup.home_team.team_name
        };

        let header = Row::new(HEADER).style(header_style());

        let away_row = Row::new(self.matchup.away_pitcher.to_row_cells(away_team));
        let home_row = Row::new(self.matchup.home_pitcher.to_row_cells(home_team));

        let widths = [
            Constraint::Length(col_width),
            Constraint::Length(20),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(4),
            Constraint::Length(5),
        ];

        let table = Table::new(vec![away_row, home_row], widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style())
                    .padding(Padding::new(1, 1, 0, 0))
                    .title(Span::styled(
                        " Probable Pitchers ",
                        Style::default().fg(Color::Black).bg(Color::Blue),
                    )),
            );

        Widget::render(table, area, buf);
    }
}
