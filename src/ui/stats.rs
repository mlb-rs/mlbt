use crate::components::stats::table::TeamOrPlayer;
use crate::components::stats::{STATS_DEFAULT_COL_WIDTH, STATS_FIRST_COL_WIDTH};
use crate::components::util::{DimColor, avg_color};
use crate::state::stats::{ActivePane, StatsState};
use mlbt_api::client::StatGroup;
use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table, Wrap};

pub const STATS_OPTIONS_WIDTH: u16 = 36;

/// Renders the stats data table (left pane).
pub struct StatsDataWidget {}

impl StatefulWidget for StatsDataWidget {
    type State = StatsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let table = state.generate_table();
        let (col_names, _, rows) = table.as_ref();

        // use the sort column to include up/down arrow in the column name
        let sort_column = state
            .table
            .sorting
            .column_name
            .as_deref()
            .unwrap_or_default();
        let header = col_names
            .iter()
            .map(|name| {
                if name == sort_column {
                    Cell::from(format!(
                        "{name} {}",
                        state.table.sorting.order.arrow_symbol()
                    ))
                    .style(Style::default().bg(Color::Blue))
                } else {
                    Cell::from(name.as_str())
                }
            })
            .collect::<Row>()
            .height(1)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        let rows: Vec<Row> = rows
            .iter()
            .map(|row| {
                col_names
                    .iter()
                    .zip(row.iter())
                    .map(|(col_name, cell)| {
                        if col_name == "AVG" {
                            Cell::from(cell.as_str()).fg(avg_color(cell).unwrap_or(Color::White))
                        } else {
                            Cell::from(cell.as_str()).fg(cell.as_str().dim_or(Color::White))
                        }
                    })
                    .collect::<Row>()
            })
            .collect();

        // Count active columns for width constraints
        let active = state.table.columns.values().filter(|v| v.active).count();

        // Build the constraints. On first load the active will be 0, hence the check.
        let mut constraints = vec![Constraint::Length(STATS_DEFAULT_COL_WIDTH); active];
        if active == 0 {
            constraints.push(Constraint::Length(STATS_FIRST_COL_WIDTH));
        } else {
            constraints[0] = Constraint::Length(STATS_FIRST_COL_WIDTH);
        }

        let mut t = Table::new(rows, constraints)
            .header(header)
            .column_spacing(0)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::new(1, 1, 0, 0))
                    .title(Span::styled(
                        state.date_selector.format_date_border_title(),
                        Style::default().fg(Color::Black).bg(Color::Blue),
                    )),
            );
        if state.active_pane == ActivePane::Data {
            t = t.row_highlight_style(Style::default().bg(Color::Blue).fg(Color::Black));
        }

        // borders (2) + header (1) = 3 rows of overhead
        state.visible_rows = area.height.saturating_sub(3) as usize;

        StatefulWidget::render(t, area, buf, &mut state.data_state);
    }
}

/// Renders the options sidebar (right pane).
pub struct StatsOptionsWidget {}

impl StatefulWidget for StatsOptionsWidget {
    type State = StatsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let selected_style = Style::default().bg(Color::Blue).fg(Color::Black);

        let [stats_rect, options_rect] =
            Layout::vertical([Constraint::Length(4), Constraint::Percentage(100)]).areas(area);

        // hitting | pitching
        // team | player
        let (hitting_style, pitching_style) = match state.stat_type.group {
            StatGroup::Pitching => (Style::default(), selected_style),
            StatGroup::Hitting => (selected_style, Style::default()),
        };
        let (team_style, player_style) = match state.stat_type.team_player {
            TeamOrPlayer::Player => (Style::default(), selected_style),
            TeamOrPlayer::Team => (selected_style, Style::default()),
        };
        let text = vec![
            Line::from(vec![
                Span::styled("hitting", hitting_style),
                Span::raw(" | "),
                Span::styled("pitching", pitching_style),
            ]),
            Line::from(vec![
                Span::styled("team", team_style),
                Span::raw(" | "),
                Span::styled("player", player_style),
            ]),
        ];
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .render(stats_rect, buf);

        // Create the options rows, e.g. ["[X]", "ERA", "earned run average"]
        let mut options = Vec::new();
        for (name, stat) in &state.table.columns {
            let selected = if stat.active { "[X]" } else { "[ ]" };
            options.push(Row::new(vec![
                selected,
                name.as_str(),
                stat.description.as_str(),
            ]));
        }

        let widths = [
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Length(25),
        ];
        let mut t = Table::new(options, widths).column_spacing(0).block(
            Block::default()
                .padding(Padding::new(1, 1, 0, 0))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
        if state.active_pane == ActivePane::Options {
            t = t.row_highlight_style(selected_style);
        }
        StatefulWidget::render(t, options_rect, buf, &mut state.options_state);
    }
}
