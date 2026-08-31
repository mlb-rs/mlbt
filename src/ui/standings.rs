use crate::components::standings::{ClinchStatus, Standing};
use crate::state::standings::{StandingsState, ViewMode};
use crate::ui::styling::{TEXT_COLOR, dim_style, win_pct_color};
use crate::ui::styling::{border_style, header_style, selected_style};
use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Cell, Padding, Row, Table};

/// The elimination number column, only shown towards the end of the season.
const ELIMINATION_HEADER: &str = "E#";
const ELIMINATION_WIDTH: Constraint = Constraint::Length(4);
const ELIMINATION_INDEX: usize = 6;
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
        let elimination_column = state.elimination_column();

        let mut header_cells: Vec<Cell> = HEADER.iter().map(|h| Cell::from(*h)).collect();
        let mut widths: Vec<Constraint> = WIDTHS.to_vec();
        if elimination_column.is_some() {
            header_cells.insert(ELIMINATION_INDEX, Cell::from(ELIMINATION_HEADER));
            widths.insert(ELIMINATION_INDEX, ELIMINATION_WIDTH);
        }
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
                        rows.push(Row::new(standing_cells(s, elimination_column)).height(1))
                    }
                }
            }
            ViewMode::Overall => {
                // Show all teams sorted by record without division headers
                for t in &state.league_standings {
                    rows.push(Row::new(standing_cells(t, elimination_column)).height(1));
                }
            }
        }

        let t = Table::new(rows, widths)
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

/// The elimination number the given view is showing.
fn elimination_cell(clinch: &ClinchStatus, view: ViewMode) -> Cell<'static> {
    let (eliminated, number) = match view {
        ViewMode::WildCard => (clinch.eliminated, clinch.wild_card_elimination_number),
        _ => (clinch.division_eliminated, clinch.elimination_number),
    };

    match (eliminated, number) {
        // dimmed to match the clinch marker
        (true, _) => Cell::from("E").style(dim_style()),
        (false, Some(number)) => Cell::from(number.to_string()),
        (false, None) => Cell::from("-"),
    }
}

/// The team name with the clinch marker appended once they have clinched.
fn team_name_cell(standing: &Standing) -> Cell<'static> {
    let Some(marker) = standing.clinch.marker() else {
        return Cell::from(standing.team.name.to_string());
    };

    Cell::from(Line::from(vec![
        Span::raw(format!("{} ", standing.team.name)),
        Span::styled(marker.to_string(), dim_style()),
    ]))
}

/// Build the table cells for one team. `elimination_column` is the view whose number to show, or
/// `None` to leave the column off.
fn standing_cells(standing: &Standing, elimination_column: Option<ViewMode>) -> Vec<Cell<'_>> {
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
    let mut cells: Vec<Cell> = vec![
        team_name_cell(standing),
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
    ];

    if let Some(view) = elimination_column {
        cells.insert(ELIMINATION_INDEX, elimination_cell(&standing.clinch, view));
    }

    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_elimination_column_lines_up_with_the_header() {
        assert_eq!(HEADER.len(), WIDTHS.len());

        // the index applies to the header, the widths and the cells alike
        assert_eq!(HEADER[ELIMINATION_INDEX - 1], "WCGB");
        assert_eq!(HEADER[ELIMINATION_INDEX], "L10");

        let standing = Standing::default();
        assert_eq!(standing_cells(&standing, None).len(), HEADER.len());

        // no other column renders a bare `-`, so this pins the cell to the header's index
        let cells = standing_cells(&standing, Some(ViewMode::ByDivision));
        assert_eq!(cells.len(), HEADER.len() + 1);
        assert_eq!(cells[ELIMINATION_INDEX], Cell::from("-"));
    }
}
