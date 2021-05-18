mod app;
mod banner;
mod boxscore;
mod debug;
#[allow(dead_code)]
mod event;
mod heatmap;
mod matchup;
mod pitches;
mod schedule;
mod ui;

use crate::app::{App, DebugState, MenuItem};
use crate::boxscore::BoxScore;
use crate::debug::DebugInfo;
use crate::event::{Event, Events};
use crate::schedule::StatefulSchedule;
use crate::ui::{help::render_help, layout::LayoutAreas, tabs::render_top_bar};
use mlb_api::MLBApiBuilder;

use crate::heatmap::Heatmap;
use crate::matchup::Matchup;
use crate::pitches::Pitches;
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
    let mut schedule_table = StatefulSchedule::new(&schedule);

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let mut app = App {
        api: &mlb,
        layout: LayoutAreas::new(terminal.size().unwrap()), // TODO don't unwrap this?
        tabs: vec!["Scoreboard", "GameDay", "Stats", "Standings"],
        active_tab: MenuItem::Scoreboard,
        previous_state: MenuItem::Scoreboard,
        schedule: &mut schedule_table,
        debug_state: DebugState::Off,
    };

    loop {
        terminal.draw(|f| {
            app.layout.update(f.size());
            render_top_bar(f, &app);

            let tempblock = Block::default().borders(Borders::ALL);
            match app.active_tab {
                MenuItem::Scoreboard => {
                    // Create block for rendering boxscore and schedule
                    let main = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(7), Constraint::Percentage(100)].as_ref())
                        .split(app.layout.main);

                    // Hit the API to update the schedule
                    app.update_schedule();
                    app.schedule.render(f, main[1]);

                    // Hit the API to get live game data TODO add error handling
                    let game_id = app.schedule.get_selected_game();
                    let live_game = app.api.get_live_data(game_id);
                    let boxscore = BoxScore::new(&live_game.live_data.linescore);
                    boxscore.render(f, main[0]);
                }
                MenuItem::GameDay => {
                    let gamedayp = Paragraph::new("").block(tempblock.clone());
                    f.render_widget(gamedayp, app.layout.main);

                    let game_id = app.schedule.get_selected_game();
                    let live_game = app.api.get_live_data(game_id);
                    let heatmap = Heatmap::from_live_data(&live_game);
                    heatmap.render(f, app.layout.main);

                    let matchup = Matchup::from_live_data(&live_game);
                    matchup.render(f, app.layout.main);

                    let test = Pitches::from_live_data(&live_game);
                    test.render(f, app.layout.main);
                }
                MenuItem::Stats => {
                    let gameday = Paragraph::new("stats").block(tempblock.clone());
                    f.render_widget(gameday, app.layout.main);
                }
                MenuItem::Standings => {
                    let gameday = Paragraph::new("standings").block(tempblock.clone());
                    f.render_widget(gameday, app.layout.main);
                }
                MenuItem::Help => render_help(f),
            }
            if app.debug_state == DebugState::On {
                let mut dbi = DebugInfo::new();
                dbi.gather_info(f, &app);
                dbi.render(f, app.layout.main)
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

                Key::Char('d') => app.toggle_debug(),
                _ => {}
            }
        };
    }
    Ok(())
}
