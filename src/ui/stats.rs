use crate::components::stats::table::TeamOrPlayer;
use crate::components::stats::{STATS_DEFAULT_COL_WIDTH, STATS_FIRST_COL_WIDTH};
use crate::components::util::{DIM_COLOR, DimColor, avg_color_or_default, era_color_or_default};
use crate::state::stats::{ActivePane, StatsState};
use mlbt_api::client::{Qualification, StatGroup};
use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table};

pub const STATS_OPTIONS_WIDTH: u16 = 36;
const HIGHLIGHT_COLOR: Color = Color::Blue;
const HIGHLIGHT_STYLE: Style = Style::new().bg(HIGHLIGHT_COLOR).fg(Color::Black);

/// Renders the stats data table (left pane).
pub struct StatsDataWidget {}

impl StatefulWidget for StatsDataWidget {
    type State = StatsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let table = state.generate_table();
        let (header, _, rows) = table.as_ref();

        let mut avg_idx = None;
        let mut era_idx = None;
        for (i, name) in header.iter().enumerate() {
            match name.as_str() {
                "AVG" => avg_idx = Some(i),
                "ERA" => era_idx = Some(i),
                _ => {}
            }
        }

        // use the sort column to include up/down arrow in the column name
        let sort_column = state
            .table
            .sorting
            .column_name
            .as_deref()
            .unwrap_or_default();
        let header = header
            .iter()
            .map(|name| {
                if name == sort_column {
                    Cell::from(format!(
                        "{name} {}",
                        state.table.sorting.order.arrow_symbol()
                    ))
                    .bg(HIGHLIGHT_COLOR)
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
                row.iter()
                    .enumerate()
                    .map(|(i, cell)| {
                        let color = if Some(i) == avg_idx {
                            avg_color_or_default(cell)
                        } else if Some(i) == era_idx {
                            era_color_or_default(cell)
                        } else {
                            cell.as_str().dim_or_default()
                        };
                        Cell::from(cell.as_str()).fg(color)
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
                        HIGHLIGHT_STYLE,
                    )),
            );
        if state.active_pane == ActivePane::Data {
            t = t.row_highlight_style(HIGHLIGHT_STYLE);
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
        let [stats_rect, options_rect] =
            Layout::vertical([Constraint::Length(5), Constraint::Percentage(100)]).areas(area);

        // hitting | pitching
        //    team | player
        //     all | qualified
        let (hitting_style, pitching_style) = match state.stat_type.group {
            StatGroup::Pitching => (Style::default(), HIGHLIGHT_STYLE),
            StatGroup::Hitting => (HIGHLIGHT_STYLE, Style::default()),
        };
        let (team_style, player_style) = match state.stat_type.team_player {
            TeamOrPlayer::Player => (Style::default(), HIGHLIGHT_STYLE),
            TeamOrPlayer::Team => (HIGHLIGHT_STYLE, Style::default()),
        };

        let is_team = state.stat_type.team_player == TeamOrPlayer::Team;
        let dim_if_team = if is_team {
            Style::default().fg(DIM_COLOR)
        } else {
            Style::default()
        };
        let (all_style, qualified_style) = if is_team {
            // disable qualification selection when Team is selected because it's not applicable
            (dim_if_team, dim_if_team)
        } else {
            match state.stat_type.qualification {
                Qualification::All => (HIGHLIGHT_STYLE, Style::default()),
                Qualification::Qualified => (Style::default(), HIGHLIGHT_STYLE),
            }
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner = block.inner(stats_rect);
        block.render(stats_rect, buf);

        // split into three chunks so that the center line is always exactly in the center
        let [left_area, divider_area, right_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(inner);

        Paragraph::new(vec![
            Line::from(Span::styled("hitting", hitting_style)),
            Line::from(Span::styled("team", team_style)),
            Line::from(Span::styled("all", all_style)),
        ])
        .alignment(Alignment::Right)
        .render(left_area, buf);

        Paragraph::new(vec![
            Line::from(" | "),
            Line::from(" | "),
            Line::from(" | ").style(dim_if_team),
        ])
        .alignment(Alignment::Center)
        .render(divider_area, buf);

        Paragraph::new(vec![
            Line::from(Span::styled("pitching", pitching_style)),
            Line::from(Span::styled("player", player_style)),
            Line::from(Span::styled("qualified", qualified_style)),
        ])
        .alignment(Alignment::Left)
        .render(right_area, buf);

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
            t = t.row_highlight_style(HIGHLIGHT_STYLE);
        }
        StatefulWidget::render(t, options_rect, buf, &mut state.options_state);
    }
}
