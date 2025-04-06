use mlb_api::client::StatGroup;

use crate::components::stats::{
    STATS_DEFAULT_COL_WIDTH, STATS_FIRST_COL_WIDTH, StatsState, TeamOrPlayer,
};

use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Paragraph, Row, StatefulWidget, Table, Widget, Wrap,
    },
};

pub const STATS_OPTIONS_WIDTH: u16 = 36;

pub struct StatsWidget {
    pub show_options: bool,
}

impl StatefulWidget for StatsWidget {
    type State = StatsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let constraints = match self.show_options {
            true => {
                vec![
                    Constraint::Length(area.width - STATS_OPTIONS_WIDTH), // stats
                    Constraint::Length(STATS_OPTIONS_WIDTH),              // options
                ]
            }
            false => vec![Constraint::Percentage(100)],
        };
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints::<&[Constraint]>(constraints.as_ref())
            .split(area);

        let (header, rows) = state.generate_table();

        // use the sort column to include up/down arrow in the column name
        let sort_column = state.sorting.column_name.as_deref().unwrap_or_default();
        let header = header
            .into_iter()
            .map(|name| {
                if name == sort_column {
                    Cell::from(format!("{name} {}", state.sorting.order.arrow_symbol()))
                        .style(Style::default().bg(Color::Blue))
                } else {
                    Cell::from(name)
                }
            })
            .collect::<Row>()
            .height(1)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        let rows: Vec<Row> = rows.into_iter().map(Row::new).collect();

        // Create the options rows, e.g. ["[X]", "ERA", "earned run average"]
        let mut active = 0;
        let mut options = Vec::new();
        for (name, stat) in &state.stats {
            let selected = match stat.active {
                true => {
                    active += 1;
                    "[X]"
                }
                false => "[ ]",
            };
            options.push(Row::new(vec![
                selected.to_string(),
                name.clone(),
                stat.description.clone(),
            ]));
        }

        // Build the constraints. On first load the active will be 0, hence the check.
        let mut constraints = vec![Constraint::Length(STATS_DEFAULT_COL_WIDTH); active];
        if active == 0 {
            constraints.push(Constraint::Length(STATS_FIRST_COL_WIDTH));
        } else {
            constraints[0] = Constraint::Length(STATS_FIRST_COL_WIDTH);
        }

        // stats
        let t = Table::new(rows, constraints)
            .header(header)
            .column_spacing(0)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );

        StatefulWidget::render(t, chunks[0], buf, &mut state.state);

        if self.show_options {
            let selected_style = Style::default().bg(Color::Blue).fg(Color::Black);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Percentage(100)].as_ref())
                .split(chunks[1]);
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
                .render(chunks[0], buf);

            // options
            let widths = [
                Constraint::Length(4),
                Constraint::Length(6),
                Constraint::Length(25),
            ];
            let t = Table::new(options, widths)
                .column_spacing(0)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .row_highlight_style(selected_style);
            StatefulWidget::render(t, chunks[1], buf, &mut state.state);
        }
    }
}
