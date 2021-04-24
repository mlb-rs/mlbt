use std::error::Error;
use std::io;

use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::app::{App, MenuItem};
use crate::event::{Event, Events};
use crate::help::render_help;

mod app;
#[allow(dead_code)]
mod event;
mod help;
mod tabs;

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
        previous_state: MenuItem::Scoreboard,
    };

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Percentage(100)].as_ref())
                .split(f.size());

            tabs::render_top_bar(f, &app, chunks[0]);

            let tempblock = Block::default().borders(Borders::ALL);
            match app.active_tab {
                MenuItem::Scoreboard => {
                    // Create block for rendering boxscore and schedule
                    let main = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(7), Constraint::Percentage(100)].as_ref())
                        .split(chunks[1]);

                    let boxscore = Paragraph::new("boxscore").block(tempblock.clone());
                    let schedule = Paragraph::new("schedule").block(tempblock.clone());
                    f.render_widget(boxscore, main[0]);
                    f.render_widget(schedule, main[1]);
                }
                MenuItem::GameDay => {
                    let gameday = Paragraph::new("gameday").block(tempblock.clone());
                    f.render_widget(gameday, chunks[1]);
                }
                MenuItem::Stats => {
                    let gameday = Paragraph::new("stats").block(tempblock.clone());
                    f.render_widget(gameday, chunks[1]);
                }
                MenuItem::Standings => {
                    let gameday = Paragraph::new("standings").block(tempblock.clone());
                    f.render_widget(gameday, chunks[1]);
                }
                MenuItem::Help => {
                    app.previous_state = app.active_tab;
                    render_help(f)
                }
            }
        })?;

        if let Event::Input(key) = events.next()? {
            match key {
                Key::Char('q') => break,

                Key::Char('1') => app.update_tab(MenuItem::Scoreboard),
                Key::Char('2') => app.update_tab(MenuItem::GameDay),
                Key::Char('3') => app.update_tab(MenuItem::Stats),
                Key::Char('4') => app.update_tab(MenuItem::Standings),

                Key::Char('?') => app.update_tab(MenuItem::Help),
                Key::Esc => app.exit_help(),
                _ => {}
            }
        };
    }
    Ok(())
}
