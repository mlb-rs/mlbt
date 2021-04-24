use std::error::Error;
use std::io;

use crate::event::{Event, Events};
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, Paragraph, Tabs},
    Terminal,
};

#[allow(dead_code)]
mod event;

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Scoreboard,
    GameDay,
    Stats,
    Standings,
}

struct App<'a> {
    tabs: Vec<&'a str>,
    active_tab: MenuItem,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let mut app = App {
        tabs: vec!["Scoreboard", "GameDay", "Stats", "Standings"],
        active_tab: MenuItem::Scoreboard,
    };

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Percentage(100)].as_ref())
                .split(f.size());

            let titles = app.tabs.iter().map(|t| Spans::from(*t)).collect();
            let tabs = Tabs::new(titles)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::White));
            f.render_widget(tabs, chunks[0]);

            match app.active_tab {
                MenuItem::Scoreboard => {
                    // Create block for rendering boxscore and schedule
                    let main = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(7), Constraint::Percentage(100)].as_ref())
                        .split(chunks[1]);

                    let boxscore = Paragraph::new("boxscore");
                    let schedule = Paragraph::new("schedule");
                    f.render_widget(boxscore, main[0]);
                    f.render_widget(schedule, main[1]);
                }
                MenuItem::GameDay => {
                    let gameday = Paragraph::new("gameday");
                    f.render_widget(gameday, chunks[1]);
                }
                _ => unimplemented!(),
            }
        })?;

        if let Event::Input(key) = events.next()? {
            match key {
                Key::Char('q') => break,

                Key::Char('1') => app.active_tab = MenuItem::Scoreboard,
                Key::Char('2') => app.active_tab = MenuItem::GameDay,
                Key::Char('3') => app.active_tab = MenuItem::Stats,
                Key::Char('4') => app.active_tab = MenuItem::Standings,

                _ => {}
            }
        };
    }
    Ok(())
}
