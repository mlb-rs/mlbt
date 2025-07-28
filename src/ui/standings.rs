use crate::components::standings::{StandingsState, ViewMode};
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
        let header = Row::new(header_cells)
            .height(1)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        // Get teams data if in overall mode to avoid borrowing conflicts
        let teams_by_record = if state.view_mode == ViewMode::Overall {
            Some(state.get_teams_by_record())
        } else {
            None
        };

        let mut rows = Vec::with_capacity(36); // 30 teams + 6 divisions
        
        match state.view_mode {
            ViewMode::ByDivision => {
                for d in &state.standings {
                    // create a row for the division name
                    let division = Row::new(vec![d.name.clone()])
                        .height(1)
                        .style(Style::default().add_modifier(Modifier::BOLD));
                    rows.push(division);
                    // then add all the teams in the division
                    for s in &d.standings {
                        rows.push(Row::new(s.to_cells()).height(1))
                    }
                }
            }
            ViewMode::Overall => {
                // Show all teams sorted by record without division headers
                if let Some(teams) = &teams_by_record {
                    for s in teams {
                        rows.push(Row::new(s.to_cells()).height(1));
                    }
                }
            }
        }

        let selected_style = Style::default().bg(Color::Blue).fg(Color::Black);
        let t = Table::new(rows, WIDTHS)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::new(1, 1, 0, 0))
                    .title(Span::styled(
                        format!("{} - {}", 
                            state.date_selector.format_date_border_title(),
                            match state.view_mode {
                                ViewMode::ByDivision => "By Division (Tab: Overall)",
                                ViewMode::Overall => "Overall (Tab: By Division)",
                            }
                        ),
                        Style::default().fg(Color::Black).bg(Color::Blue),
                    )),
            )
            .row_highlight_style(selected_style);

        StatefulWidget::render(t, area, buf, &mut state.state);
    }
}
