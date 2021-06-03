#![allow(unused_imports)] // temp
mod app;
mod at_bat;
mod banner;
mod boxscore;
mod boxscore_stats;
mod debug;
mod draw;
mod event;
mod gameday;
mod matchup;
mod pitches;
mod plays;
mod schedule;
mod strikezone;
mod ui;
mod util;

use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread};

use crossbeam_channel::{bounded, select, unbounded, Receiver, Sender};

use crate::app::{App, BoxscoreTab, DebugState, MenuItem};
use crate::boxscore::BoxScore;
use crate::debug::DebugInfo;
use crate::gameday::Gameday;
use crate::schedule::StatefulSchedule;
use crate::ui::{help::render_help, layout::LayoutAreas, tabs::render_top_bar};

use mlb_api::client::MLBApiBuilder;

use termion::event::Event;
use termion::input::TermRead;
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

extern crate chrono;
extern crate chrono_tz;
#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref REDRAW_REQUEST: (Sender<()>, Receiver<()>) = bounded(1);
    pub static ref DATA_RECEIVED: (Sender<()>, Receiver<()>) = bounded(1);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mlb = MLBApiBuilder::default().build().unwrap();

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // let events = Events::new();
    let request_redraw = REDRAW_REQUEST.0.clone();
    let data_received = DATA_RECEIVED.1.clone();
    let ui_events = setup_ui_events();

    let app = Arc::new(Mutex::new(App {
        active_tab: MenuItem::Scoreboard,
        previous_state: MenuItem::Scoreboard,
        // gameday: &mut Gameday::new(),
        // schedule: &mut StatefulSchedule::new(&mlb.get_todays_schedule()),
        debug_state: DebugState::Off,
        boxscore_tab: BoxscoreTab::Home,
    }));
    let move_app = app.clone();

    // Redraw thread
    thread::spawn(move || {
        let app = move_app;

        let redraw_requested = REDRAW_REQUEST.1.clone();

        loop {
            select! {
                recv(redraw_requested) -> _ => {
                    let mut app = app.lock().unwrap();

                    draw::draw(&mut terminal, &mut app);
                }
                // Default redraw on every duration
                default(Duration::from_millis(500)) => {
                    let mut app = app.lock().unwrap();

                    draw::draw(&mut terminal, &mut app);
                }
            }
        }
    });

    loop {
        select! {
            // Notified that new data has been fetched from API, update widgets
            // so they can update their state with this new information
            recv(data_received) -> _ => {
                let mut app = app.lock().unwrap();

                // app.update();
            }
            recv(ui_events) -> message => {
                let mut app = app.lock().unwrap();

                match message {
                    Ok(Event::Key(key_event)) => {
                            event::handle_key_bindings(app.active_tab, key_event, &mut app, &request_redraw);
                    }
                    // Ok(Event::Resize(..)) => {
                    //     let _ = request_redraw.try_send(());
                    // }
                    _ => {}
                }
            }
        }
    }
}

fn cleanup_terminal() {
    // terminal.clear();
    // let mut stdout = io::stdout();
    // execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
    // execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
    // terminal::disable_raw_mode().unwrap();
}

fn setup_ui_events() -> Receiver<Event> {
    let (sender, receiver) = unbounded();
    std::thread::spawn(move || loop {
        let stdin = io::stdin();
        for evt in stdin.keys().flatten() {
            if let Err(err) = sender.send(termion::event::Event::Key(evt)) {
                eprintln!("{}", err);
                return;
            }
        }
    });

    receiver
}
