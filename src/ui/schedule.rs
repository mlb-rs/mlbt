use crate::components::schedule::{Record, ScheduleRow, ScheduleState};
use crate::state::app_state::HomeOrAway;
use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Cell, Padding, Row, Table};

const HEADER: &[&str; 8] = &["away", "", "", "home", "", "", "time", "status"];

pub struct ScheduleWidget {
    pub tz_abbreviation: String,
}

impl ScheduleRow {
    const ABBREVIATION_WIDTH: u16 = 70;

    fn format_record(record: Option<Record>) -> String {
        record
            .map(|r| r.to_display_string())
            .unwrap_or(Record::default_display_string())
    }

    fn default_score(score: Option<u8>) -> String {
        let s = score
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        format!("{s:<3}")
    }

    fn get_styles(&self, team: HomeOrAway) -> (Style, Style) {
        let winning_team = self.winning_team();
        let lose_style = Style::default().fg(Color::DarkGray);
        match winning_team {
            Some(winner) if winner == team => (Style::default(), Style::default()),
            None => (Style::default(), Style::default()),
            _ => (lose_style, lose_style),
        }
    }

    fn format(&self, width: u16) -> Vec<Span<'_>> {
        let (away_team_style, away_score_style) = self.get_styles(HomeOrAway::Away);
        let (home_team_style, home_score_style) = self.get_styles(HomeOrAway::Home);
        let away_record = Self::format_record(self.away_record);
        let home_record = Self::format_record(self.home_record);

        let (away_team, home_team) = if width < Self::ABBREVIATION_WIDTH {
            (self.away_team.abbreviation, self.home_team.abbreviation)
        } else {
            (self.away_team.team_name, self.home_team.team_name)
        };

        vec![
            Span::styled(away_team, away_team_style),
            Span::styled(away_record, away_team_style),
            Span::styled(Self::default_score(self.away_score), away_score_style),
            Span::styled(home_team, home_team_style),
            Span::styled(home_record, home_team_style),
            Span::styled(Self::default_score(self.home_score), home_score_style),
            Span::raw(self.start_time.to_string()),
            Span::raw(self.game_status.to_string()),
        ]
    }
}

impl StatefulWidget for ScheduleWidget {
    type State = ScheduleState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header_cells = HEADER.iter().enumerate().map(|(i, h)| {
            if i == 6 {
                Cell::from(format!("{} [{}]", *h, self.tz_abbreviation))
            } else {
                Cell::from(*h)
            }
        });

        let header = Row::new(header_cells)
            .height(1)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        let rows = state
            .schedule
            .iter()
            .map(|r| Row::new(r.format(area.width)));
        let name_constraint = if area.width < ScheduleRow::ABBREVIATION_WIDTH {
            Constraint::Length(5)
        } else {
            Constraint::Length(11)
        };
        let widths = [
            name_constraint,        // away team name
            Constraint::Length(6),  // away team record
            Constraint::Length(3),  // away score
            name_constraint,        // home team name
            Constraint::Length(6),  // home team record
            Constraint::Length(3),  // home score
            Constraint::Length(12), // game time
            Constraint::Fill(1),    // game status
        ];
        let selected_style = Style::default().bg(Color::Blue).fg(Color::Black);

        let t = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::new(1, 1, 0, 0))
                    .title(Span::styled(
                        state.date_selector.format_date_border_title(),
                        Style::default().fg(Color::Black).bg(Color::Blue),
                    )),
            )
            .row_highlight_style(selected_style);

        StatefulWidget::render(t, area, buf, &mut state.state);
    }
}
