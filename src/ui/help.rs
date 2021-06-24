use tui::layout::Alignment;
use tui::widgets::Paragraph;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, Row, Table},
    Frame,
};

use crate::banner::BANNER;

pub fn get_help_docs() -> Vec<Vec<String>> {
    vec![
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

// based on: https://github.com/Rigellute/spotify-tui/blob/master/src/ui/mod.rs#L76
pub fn render_help<B>(f: &mut Frame<B>)
where
    B: Backend,
{
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

    let header = ["Description", "Key"];
    let header = format_row(header.iter().map(|s| s.to_string()).collect());
    let header = Row::new(header.text)
        .height(1)
        .bottom_margin(0)
        .style(header_style);

    let help_docs = get_help_docs();
    let help_docs = help_docs
        .into_iter()
        .map(format_row)
        .collect::<Vec<HelpRow>>();
    let rows = help_docs.iter().map(|item| match item.is_header {
        true => Row::new(item.text.clone()).style(header_style),
        false => Row::new(item.text.clone()).style(help_menu_style),
    });

    let help_menu = Table::new(rows)
        .widths(&[Constraint::Max(50)])
        .header(header)
        .style(help_menu_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled("Help - press <Esc> to exit", help_menu_style)),
        );

    let vert = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Length(15)].as_ref())
        .split(f.size());

    let logo = Paragraph::new(format!("{}\nv {}", BANNER, env!("CARGO_PKG_VERSION")))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(help_menu_style),
        );

    f.render_widget(Clear, vert[0]); //this clears out the background
    f.render_widget(Clear, vert[1]); //this clears out the background
    f.render_widget(help_menu, vert[0]);
    f.render_widget(logo, vert[1]);
}
