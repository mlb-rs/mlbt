use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Paragraph, Row, Table, Widget},
};

use crate::banner::BANNER;

const HEADER: &[&str; 2] = &["Description", "Key"];

fn docs() -> Vec<Vec<String>> {
    vec![
        vec!["Exit help".to_string(), "Esc".to_string()],
        vec!["Quit".to_string(), "q".to_string()],
        vec!["Scoreboard".to_string(), "1".to_string()],
        vec!["Gameday".to_string(), "2".to_string()],
        vec!["Stats".to_string(), "3".to_string()],
        vec!["Standings".to_string(), "4".to_string()],
        vec!["Scoreboard".to_string(), "".to_string()],
        vec!["Move down".to_string(), "j".to_string()],
        vec!["Move up".to_string(), "k".to_string()],
        vec!["Gameday".to_string(), "".to_string()],
        vec!["Toggle game info".to_string(), "i".to_string()],
        vec!["Toggle pitches".to_string(), "p".to_string()],
        vec!["Toggle boxscore".to_string(), "b".to_string()],
        vec!["Switch boxscore team".to_string(), "h/a".to_string()],
        vec!["Standings".to_string(), "".to_string()],
        vec!["Move down".to_string(), "j".to_string()],
        vec!["Move up".to_string(), "k".to_string()],
        vec!["View team info".to_string(), "Enter".to_string()],
    ]
}

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
        let format_row = |r: Vec<String>| -> HelpRow {
            HelpRow {
                is_header: r[1].is_empty(),
                text: vec![format!("{:30}{:15}", r[0], r[1])],
            }
        };

        let header_style = Style::default().add_modifier(Modifier::BOLD);
        let help_menu_style = Style::default();

        let header = format_row(HEADER.iter().map(|s| s.to_string()).collect());
        let header = Row::new(header.text)
            .height(1)
            .bottom_margin(0)
            .style(header_style);

        let help_docs = docs().into_iter().map(format_row).collect::<Vec<HelpRow>>();
        let rows = help_docs.iter().map(|item| match item.is_header {
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
