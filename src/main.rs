mod app;
mod banner;
mod boxscore;
#[allow(dead_code)]
mod event;
mod help;
mod schedule;
mod tabs;

use crate::app::{App, MenuItem};
use crate::boxscore::render_boxscore;
use crate::event::{Event, Events};
use crate::help::render_help;
use crate::schedule::{render_schedule, StatefulSchedule};
use mlb_api::MLBApiBuilder;

use std::error::Error;
use std::io;
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), Box<dyn Error>> {
    let mlb = MLBApiBuilder::default().build().unwrap();
    let schedule = mlb.get_todays_schedule();

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let mut schedule_table = StatefulSchedule::new(&schedule);
    schedule_table.state.select(Some(0));

    let mut app = App {
        tabs: vec!["Scoreboard", "GameDay", "Stats", "Standings"],
        active_tab: MenuItem::Scoreboard,
        previous_state: MenuItem::Scoreboard,
        schedule: &mut schedule_table,
        api: &mlb,
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

                    // Hit the API to update the schedule
                    app.update_schedule();
                    render_schedule(f, main[1], &mut app);

                    // Hit the API to get live game data TODO add error handling
                    let game_id = app.schedule.get_selected_game();
                    let live_game = app.api.get_live_data(game_id);
                    render_boxscore(f, main[0], &live_game.live_data.linescore);
                    render_boxscore(f, main[0], &game_data);
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
                MenuItem::Help => render_help(f),
            }
        })?;

        if let Event::Input(key) = events.next()? {
            match key {
                Key::Char('q') => break,

                Key::Char('1') => app.update_tab(MenuItem::Scoreboard),
                Key::Char('2') => app.update_tab(MenuItem::GameDay),
                Key::Char('3') => app.update_tab(MenuItem::Stats),
                Key::Char('4') => app.update_tab(MenuItem::Standings),

                Key::Char('j') => app.schedule.next(),
                Key::Char('k') => app.schedule.previous(),

                Key::Char('?') => app.update_tab(MenuItem::Help),
                Key::Esc => app.exit_help(),
                _ => {}
            }
        };
    }
    Ok(())
}
