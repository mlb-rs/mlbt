use crate::stats::{StatsState, STATS_DEFAULT_COL_WIDTH, STATS_FIRST_COL_WIDTH};

use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table},
};

pub const STATS_OPTIONS_WIDTH: u16 = 35;
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
            .constraints(constraints.as_ref())
            .split(area);

        let (header, rows) = state.generate_table();

        let header = Row::new(header)
            .height(1)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        // TODO see if possible to remove another iter and clone here
        let rows: Vec<Row> = rows.iter().map(|r| Row::new(r.iter().cloned())).collect();

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
        let t = Table::new(rows)
            .header(header)
            .column_spacing(0)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .widths(constraints.as_ref());

        StatefulWidget::render(t, chunks[0], buf, &mut state.state);

        // options
        if self.show_options {
            let selected_style = Style::default().bg(Color::Blue).fg(Color::Black);
            let t = Table::new(options)
                .column_spacing(0)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .highlight_style(selected_style)
                .widths(&[
                    Constraint::Length(4),
                    Constraint::Length(5),
                    Constraint::Length(25),
                ]);
            StatefulWidget::render(t, chunks[1], buf, &mut state.state);
        }
    }
}
