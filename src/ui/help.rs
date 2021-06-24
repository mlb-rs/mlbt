use tui::layout::Alignment;
use tui::widgets::Paragraph;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, Row, Table},
    Frame,
};

use crate::banner::BANNER;

fn get_help_docs() -> Vec<Vec<String>> {
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
pub fn draw_help<B>(f: &mut Frame<B>)
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

    let help_docs = get_help_docs()
        .into_iter()
        .map(format_row)
        .collect::<Vec<HelpRow>>();
    let rows = help_docs.iter().map(|item| match item.is_header {
        true => Row::new(item.text.clone()).style(header_style),
        false => Row::new(item.text.clone()).style(help_menu_style),
    });

    // TODO test these on different terminals
    // if the terminal height is too small hide the logo
    let constraints = match f.size().height {
        w if w < 29 => [Constraint::Percentage(100), Constraint::Length(0)],
        _ => [Constraint::Percentage(70), Constraint::Length(15)],
    };

    // if the terminal is too small display a red border
    let mut border_style = Style::default();
    if f.size().height < 20 || f.size().width < 35 {
        border_style = border_style.fg(Color::Red);
    }

    let help_menu = Table::new(rows)
        .widths(&[Constraint::Max(50)])
        .header(header)
        .style(help_menu_style)
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .title(Span::styled("Help - press <Esc> to exit", help_menu_style)),
        );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.as_ref())
        .split(f.size());

    let logo = Paragraph::new(format!("{}\nv {}", BANNER, env!("CARGO_PKG_VERSION")))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_type(BorderType::Rounded)
                .style(help_menu_style),
        );

    f.render_widget(Clear, f.size());
    f.render_widget(help_menu, chunks[0]);
    f.render_widget(logo, chunks[1]);
}
