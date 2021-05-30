use crate::app::App;
use crate::boxscore_stats::TeamBatterBoxscore;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, Table, Tabs},
    Frame,
};

const HEADER: [&'static str; 9] = ["player", "ab", "r", "h", "rbi", "bb", "so", "lob", "avg"];

impl TeamBatterBoxscore {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect, app: &App)
    where
        B: Backend,
    {
        let chunk = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(2)
            .vertical_margin(1)
            .constraints(
                [
                    Constraint::Length(10),
                    Constraint::Length(3),
                    Constraint::Percentage(70),
                ]
                .as_ref(),
            )
            .split(rect);

        let tabs = Tabs::new(
            app.boxscore_tabs
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::White),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect(),
        )
        .block(Block::default().borders(Borders::NONE))
        .select(app.get_boxscore_tab())
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Blue),
        );

        f.render_widget(tabs, chunk[1]);

        let width = 3;
        let mut widths = vec![Constraint::Length(width); HEADER.len()];
        // the first width needs to be wider to display the team name
        widths[0] = Constraint::Length(15);
        // the last width needs to be wider to display batting average
        widths[HEADER.len() - 1] = Constraint::Length(5);

        let header = Row::new(HEADER.iter().map(|h| Cell::from(*h)).collect::<Vec<Cell>>())
            .height(1)
            .bottom_margin(1)
            .style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            );

        let t = Table::new(
            self.to_table_row(&app.boxscore_tab)
                .iter()
                .map(|row| Row::new(row.clone())),
        )
        .widths(widths.as_slice())
        .column_spacing(1)
        .style(Style::default().fg(Color::White))
        .header(header)
        .block(Block::default().borders(Borders::NONE));

        f.render_widget(t, chunk[2]);
    }
}
