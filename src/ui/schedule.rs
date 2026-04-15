use crate::components::schedule::{Record, ScheduleRow, ScheduleState};
use crate::components::standings::Team;
use crate::components::team_colors;
use crate::state::app_state::HomeOrAway;
use crate::symbols::Symbols;
use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Cell, Padding, Row, Table};

const HEADER: &[&str; 8] = &["away", "", "", "home", "", "", "time", "status"];

pub struct ScheduleWidget<'a> {
    pub tz_abbreviation: String,
    pub symbols: &'a Symbols,
    pub favorite_team: Option<Team>,
}

impl ScheduleRow {
    pub const ABBREVIATION_WIDTH: u16 = 70;
    pub const ABBREVIATION_COL_WIDTH: u16 = 5;
    pub const NORMAL_COL_WIDTH: u16 = 11;

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

    fn format<'a>(&'a self, width: u16, symbols: &Symbols, favorite_team: Option<Team>) -> Vec<Span<'a>> {
        let (away_team_style, away_score_style) = self.get_styles(HomeOrAway::Away);
        let (home_team_style, home_score_style) = self.get_styles(HomeOrAway::Home);
        let away_record = Self::format_record(self.away_record);
        let home_record = Self::format_record(self.home_record);

        let is_favorite = favorite_team
            .map(|t| t.id == self.away_team.id || t.id == self.home_team.id)
            .unwrap_or(false);
        let marker = if is_favorite { symbols.favorite_marker() } else { "  " };

        let (away_team, home_team) = if width < Self::ABBREVIATION_WIDTH {
            (self.away_team.abbreviation, self.home_team.abbreviation)
        } else {
            (self.away_team.team_name, self.home_team.team_name)
        };

        // Merge team color into base style when nerd_fonts is enabled
        let color_style = |base: Style, abbr: &str| -> Style {
            if symbols.nerd_fonts() {
                team_colors::get(abbr, symbols.official_team_colors())
                    .map(|c| base.fg(c))
                    .unwrap_or(base)
            } else {
                base
            }
        };

        vec![
            Span::styled(
                format!("{marker}{away_team}"),
                color_style(away_team_style, self.away_team.abbreviation),
            ),
            Span::styled(away_record, away_team_style),
            Span::styled(Self::default_score(self.away_score), away_score_style),
            Span::styled(
                home_team,
                color_style(home_team_style, self.home_team.abbreviation),
            ),
            Span::styled(home_record, home_team_style),
            Span::styled(Self::default_score(self.home_score), home_score_style),
            Span::raw(self.start_time.to_string()),
            Span::raw(self.game_status.to_string()),
        ]
    }
}

impl StatefulWidget for ScheduleWidget<'_> {
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
            .map(|r| Row::new(r.format(area.width, self.symbols, self.favorite_team)));
        let name_constraint = if area.width < ScheduleRow::ABBREVIATION_WIDTH {
            Constraint::Length(ScheduleRow::ABBREVIATION_COL_WIDTH + 2) // +2 for the always-2-char favorite marker
        } else {
            // dynamically size the team name column to fit the longest name in the schedule.
            // this accommodates longer international team names (e.g. WBC) while staying tight
            // on MLB-only days.
            let max_name_len = state
                .schedule
                .iter()
                .map(|r| r.home_team.team_name.len().max(r.away_team.team_name.len()))
                .max()
                .unwrap_or(ScheduleRow::NORMAL_COL_WIDTH as usize);
            Constraint::Length(
                (max_name_len.max(ScheduleRow::NORMAL_COL_WIDTH as usize) + 2) as u16,
            ) // +2 for the always-2-char favorite marker
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
