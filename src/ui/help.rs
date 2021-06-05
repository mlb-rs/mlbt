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

use super::utils::centered_rect;
use crate::banner::BANNER;

pub fn get_help_docs() -> Vec<Vec<String>> {
    vec![
        vec!["Quit".to_string(), "q".to_string()],
        vec!["Scoreboard".to_string(), "1".to_string()],
        vec!["Gameday".to_string(), "2".to_string()],
        vec!["Stats".to_string(), "3".to_string()],
        vec!["Standings".to_string(), "4".to_string()],
        vec!["".to_string(), "".to_string()],
        vec!["Scoreboard:".to_string(), "".to_string()],
        vec!["Move down".to_string(), "j".to_string()],
        vec!["Move up".to_string(), "k".to_string()],
        vec!["".to_string(), "".to_string()],
        vec!["Gameday:".to_string(), "".to_string()],
        vec!["Toggle game info".to_string(), "i".to_string()],
        vec!["Toggle pitches".to_string(), "p".to_string()],
        vec!["Toggle boxscore".to_string(), "b".to_string()],
        vec!["Switch boxscore team".to_string(), "h/a".to_string()],
    ]
}

// based on: https://github.com/Rigellute/spotify-tui/blob/master/src/ui/mod.rs#L76
pub fn render_help<B>(f: &mut Frame<B>)
where
    B: Backend,
{
    // Create a one-column table to avoid flickering due to non-determinism when
    // resolving constraints on widths of table columns.
    let format_row = |r: Vec<String>| -> Vec<String> { vec![format!("{:50}{:40}", r[0], r[1])] };

    let help_menu_style = Style::default();
    let header = ["Description", "Key"];
    let header = format_row(header.iter().map(|s| s.to_string()).collect());
    let header = Row::new(header)
        .height(1)
        .bottom_margin(1)
        .style(Style::default().add_modifier(Modifier::BOLD));

    let help_docs = get_help_docs();
    let help_docs = help_docs
        .into_iter()
        .map(format_row)
        .collect::<Vec<Vec<String>>>();
    let rows = help_docs
        .iter()
        .map(|item| Row::new(item.clone()).style(help_menu_style));

    let help_menu = Table::new(rows)
        .widths(&[Constraint::Max(110)])
        .header(header)
        .style(help_menu_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(help_menu_style)
                .title(Span::styled("Help - press <Esc> to exit", help_menu_style))
                .border_style(help_menu_style),
        );

    let area = centered_rect(60, 40, f.size());
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Length(15)].as_ref())
        .split(f.size());

    let logo = Paragraph::new(format!("{}\nv {}", BANNER, env!("CARGO_PKG_VERSION")))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(help_menu_style),
        );
    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(help_menu, area);
    f.render_widget(logo, vert[1]);
}
