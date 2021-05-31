mod app;
mod at_bat;
mod banner;
mod boxscore;
mod boxscore_stats;
mod debug;
#[allow(dead_code)]
mod event;
mod gameday;
mod matchup;
mod pitches;
mod plays;
mod schedule;
mod strikezone;
mod ui;
mod util;

use crate::app::{App, BoxscoreTab, DebugState, MenuItem};
use crate::boxscore::BoxScore;
use crate::debug::DebugInfo;
use crate::event::{Event, Events};
use crate::gameday::Gameday;
use crate::schedule::StatefulSchedule;
use crate::ui::{help::render_help, layout::LayoutAreas, tabs::render_top_bar};

use mlb_api::client::MLBApiBuilder;

use std::error::Error;
use std::io;
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    widgets::{Block, BorderType, Borders, Paragraph},
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
        tabs: vec!["Scoreboard", "Gameday", "Stats", "Standings"],
        active_tab: MenuItem::Gameday,
        previous_state: MenuItem::Scoreboard,
        gameday: &mut Gameday::new(),
        schedule: &mut schedule_table,
        debug_state: DebugState::Off,
        boxscore_tabs: vec!["home", "away"],
        boxscore_tab: BoxscoreTab::Home,
    };

    loop {
        terminal.draw(|f| {
            app.layout.update(f.size());
            render_top_bar(f, &app);

            let tempblock = Block::default().borders(Borders::ALL);
            match app.active_tab {
                MenuItem::Scoreboard => {
                    // Create block for rendering boxscore and schedule
                    let layout = app.layout.for_boxscore();

                    // Hit the API to update the schedule
                    app.update_schedule();
                    app.schedule.render(f, layout[1]);

                    // Hit the API to get live game data TODO add error handling
                    let game_id = app.schedule.get_selected_game();
                    let live_game = app.api.get_live_data(game_id);

                    // temp
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded);
                    f.render_widget(block, layout[0]);
                    let boxscore = BoxScore::from_live_data(&live_game);
                    boxscore.render(f, layout[0]);
                }
                MenuItem::Gameday => {
                    let game_id = app.schedule.get_selected_game();
                    let live_game = app.api.get_live_data(game_id);

                    app.gameday.load_live_data(&live_game);
                    app.gameday.render(f, app.layout.main, &app);
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
            if app.active_tab == MenuItem::Gameday {
                match key {
                    Key::Char('i') => app.gameday.toggle_info(),
                    Key::Char('p') => app.gameday.toggle_heat(),
                    Key::Char('b') => app.gameday.toggle_box(),
                    Key::Char('h') => app.boxscore_tab = BoxscoreTab::Home,
                    Key::Char('a') => app.boxscore_tab = BoxscoreTab::Away,
                    _ => {}
                }
            }
            match key {
                Key::Char('q') => break,

                Key::Char('1') => app.update_tab(MenuItem::Scoreboard),
                Key::Char('2') => app.update_tab(MenuItem::Gameday),
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
