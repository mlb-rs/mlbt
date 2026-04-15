use crate::components::standings::{StandingsState, ViewMode};
use crate::symbols::Symbols;
use crate::theme::Theme;
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

pub struct StandingsWidget<'a> {
    pub symbols: &'a Symbols,
}

impl StatefulWidget for StandingsWidget<'_> {
    type State = StandingsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header_cells = HEADER.iter().map(|h| Cell::from(*h));
        let header = Row::new(header_cells)
            .height(1)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        let mut rows = Vec::with_capacity(36); // 30 teams + 6 divisions

        match state.view_mode {
            ViewMode::ByDivision => {
                for d in &state.standings {
                    // create a row for the division name
                    let mut div_style = Style::default().add_modifier(Modifier::BOLD);
                    if self.symbols.theme().use_backgrounds() {
                        div_style = div_style.bg(Theme::ROW_HIGHLIGHT);
                    }
                    let division = Row::new(vec![d.name.clone()]).height(1).style(div_style);
                    rows.push(division);
                    // then add all the teams in the division
                    for s in &d.standings {
                        rows.push(Row::new(s.to_cells(self.symbols)).height(1))
                    }
                }
            }
            ViewMode::Overall => {
                // Show all teams sorted by record without division headers
                for t in &state.league_standings {
                    rows.push(Row::new(t.to_cells(self.symbols)).height(1));
                }
            }
        }

        let selected_style = self.symbols.theme().selection_style();
        let t = Table::new(rows, WIDTHS)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::new(1, 1, 0, 0))
                    .title(Span::styled(
                        state.date_selector.format_date_border_title(),
                        self.symbols.theme().title_style(),
                    )),
            )
            .row_highlight_style(selected_style);

        StatefulWidget::render(t, area, buf, &mut state.state);
    }
}
