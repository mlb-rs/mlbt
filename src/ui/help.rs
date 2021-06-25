use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Paragraph, Row, Table, Widget},
};

use crate::banner::BANNER;

const HEADER: &[&str; 2] = &["Description", "Key"];
pub const DOCS_LEN: usize = 19;
const DOCS: &[&[&str; 2]; DOCS_LEN] = &[
    &["Exit help", "Esc"],
    &["Quit", "q"],
    &["Scoreboard", "1"],
    &["Gameday", "2"],
    &["Stats", "3"],
    &["Standings", "4"],
    &["Scoreboard", ""],
    &["Move down", "j"],
    &["Move up", "k"],
    &["Select date", ":"],
    &["Gameday", ""],
    &["Toggle game info", "i"],
    &["Toggle pitches", "p"],
    &["Toggle boxscore", "b"],
    &["Switch boxscore team", "h/a"],
    &["Standings", ""],
    &["Move down", "j"],
    &["Move up", "k"],
    &["View team info", "Enter"],
];

/// Used to keep track of whether a row should be styled like a header.
struct HelpRow {
    is_header: bool,
    text: Vec<String>,
}

pub struct HelpWidget {}

impl Widget for HelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create a one-column table to avoid flickering due to non-determinism when
        // resolving constraints on widths of table columns.
        let format_row = |r: &[&str; 2]| -> HelpRow {
            HelpRow {
                is_header: r[1].is_empty(),
                text: vec![format!("{:30}{:15}", r[0], r[1])],
            }
        };
        let header_style = Style::default().add_modifier(Modifier::BOLD);
        let help_menu_style = Style::default();

        let header = Row::new(format_row(HEADER).text)
            .height(1)
            .bottom_margin(0)
            .style(header_style);

        let docs = DOCS
            .iter()
            .map(|d| format_row(*d))
            .collect::<Vec<HelpRow>>();

        let rows = docs.iter().map(|item| match item.is_header {
            true => Row::new(item.text.clone()).style(header_style),
            false => Row::new(item.text.clone()).style(help_menu_style),
        });

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(50), Constraint::Length(15)].as_ref())
            .margin(1)
            .split(area);

        Table::new(rows)
            .widths(&[Constraint::Max(50)])
            .header(header)
            .style(help_menu_style)
            .render(chunks[0], buf);

        Paragraph::new(format!("{}\nv {}", BANNER, env!("CARGO_PKG_VERSION")))
            .alignment(Alignment::Center)
            .render(chunks[1], buf);
    }
}
