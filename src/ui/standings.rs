use crate::components::standings::Standing;
use crate::state::standings::{StandingsState, ViewMode};
use crate::ui::styling::{TEXT_COLOR, win_pct_color};
use crate::ui::styling::{border_style, header_style, selected_style};
use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Cell, Padding, Row, Table};

const HEADER: &[&str] = &[
    "Team", "W", "L", "PCT", "GB", "WCGB", "L10", "STRK", "RS", "RA", "DIFF", "X-W/L", "HOME",
    "AWAY",
];
const WIDTHS: [Constraint; 14] = [
    Constraint::Length(25),
    Constraint::Length(5),
    Constraint::Length(5),
    Constraint::Length(5),
    Constraint::Length(5),
    Constraint::Length(6),
    Constraint::Length(5),
    Constraint::Length(5),
    Constraint::Length(5),
    Constraint::Length(5),
    Constraint::Length(6),
    Constraint::Length(8),
    Constraint::Length(8),
    Constraint::Length(8),
];

pub struct StandingsWidget {}

impl StatefulWidget for StandingsWidget {
    type State = StandingsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header_cells = HEADER.iter().map(|h| Cell::from(*h));
        let header = Row::new(header_cells).height(1).style(header_style());

        let mut rows = Vec::with_capacity(36); // 30 teams + 6 divisions

        // division is the default view, so only label the other two
        let date = state.date_selector.format_date_border_title();
        let title = match state.view_mode {
            ViewMode::ByDivision => date,
            ViewMode::Overall => format!("{date}[league] "),
            ViewMode::WildCard => format!("{date}[wild card] "),
        };

        match state.view_mode {
            // the wild card race is grouped by league instead of division
            ViewMode::ByDivision | ViewMode::WildCard => {
                let groups = if state.view_mode == ViewMode::WildCard {
                    &state.wild_card_standings
                } else {
                    &state.standings
                };
                for d in groups {
                    // create a row for the division name
                    let division = Row::new(vec![d.name.clone()])
                        .height(1)
                        .style(Style::default().add_modifier(Modifier::BOLD));
                    rows.push(division);
                    // then add all the teams in the division
                    for s in &d.standings {
                        rows.push(Row::new(standing_cells(s)).height(1))
                    }
                }
            }
            ViewMode::Overall => {
                // Show all teams sorted by record without division headers
                for t in &state.league_standings {
                    rows.push(Row::new(standing_cells(t)).height(1));
                }
            }
        }

        let t = Table::new(rows, WIDTHS)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style())
                    .padding(Padding::new(1, 1, 0, 0))
                    .title(Span::styled(title, selected_style())),
            )
            .row_highlight_style(selected_style());

        StatefulWidget::render(t, area, buf, &mut state.state);
    }
}

/// Build the table cells for one team's standings row.
fn standing_cells(standing: &Standing) -> Vec<Cell<'_>> {
    let (prefix, rdiff_color) = match standing.run_differential.signum() {
        1 => ("+", Color::Green),
        -1 => ("", Color::Red),
        _ => ("", TEXT_COLOR),
    };
    let pct_color = win_pct_color(&standing.winning_percentage);
    let streak_color = match standing.streak.chars().next() {
        Some('W') => Color::Green,
        Some('L') => Color::Red,
        _ => TEXT_COLOR,
    };
    vec![
        standing.team.name.to_string().into(),
        standing.wins.to_string().into(),
        standing.losses.to_string().into(),
        Cell::from(standing.winning_percentage.clone()).fg(pct_color),
        standing.games_back.clone().into(),
        standing.wild_card_games_back.clone().into(),
        standing.last_10.clone().into(),
        Cell::from(standing.streak.clone()).fg(streak_color),
        standing.runs_scored.to_string().into(),
        standing.runs_allowed.to_string().into(),
        Cell::from(format!("{}{}", prefix, standing.run_differential)).fg(rdiff_color),
        standing.xwl.clone().into(),
        standing.home.clone().into(),
        standing.away.clone().into(),
    ]
}
