use crate::app::HomeOrAway;
use crate::components::schedule::{ScheduleRow, ScheduleState};
use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Cell, Padding, Row, Table};

const HEADER: &[&str; 6] = &["away", "", "home", "", "time", "status"];

pub struct ScheduleWidget {
    pub tz_abbreviation: String,
}

impl ScheduleRow {
    fn format(&self) -> Vec<Span> {
        let winning_team = self.winning_team();

        fn default_score(score: Option<u8>) -> String {
            match score {
                Some(s) => s.to_string(),
                _ => "-".to_string(),
            }
        }
        let lose_style = Style::default().fg(Color::Gray);

        let away_score = match winning_team {
            Some(HomeOrAway::Away) => Span::raw(format!("{:<3}", default_score(self.away_score))),
            _ => Span::styled(format!("{:<3}", default_score(self.away_score)), lose_style),
        };

        let home_score = match winning_team {
            Some(HomeOrAway::Home) => Span::raw(format!("{:<6}", default_score(self.home_score))),
            _ => Span::styled(format!("{:<6}", default_score(self.home_score)), lose_style),
        };

        vec![
            Span::raw(format!("{:10}", self.away_team)),
            away_score,
            Span::raw(format!("{:10}", self.home_team)),
            home_score,
            Span::raw(format!("{:14}", self.start_time)),
            Span::raw(format!("{:20}", self.game_status)),
        ]
    }
}

impl StatefulWidget for ScheduleWidget {
    type State = ScheduleState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header_cells = HEADER.iter().enumerate().map(|(i, h)| {
            if i == 4 {
                Cell::from(format!("{} [{}]", *h, self.tz_abbreviation))
            } else {
                Cell::from(*h)
            }
        });

        let header = Row::new(header_cells)
            .height(1)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        let rows = state.schedule.iter().map(|r| Row::new(r.format()));
        let selected_style = Style::default().bg(Color::Blue).fg(Color::Black);
        let widths = [
            Constraint::Length(10), // away team name
            Constraint::Length(3),  // away score
            Constraint::Length(10), // home team name
            Constraint::Length(6),  // home score + padding
            Constraint::Length(14), // game time
            Constraint::Length(20), // game status
        ];

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
