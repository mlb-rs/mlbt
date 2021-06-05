mod app;
mod at_bat;
mod banner;
mod boxscore_stats;
mod debug;
mod draw;
mod event;
mod linescore;
mod live_game;
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

use crate::app::{App, DebugState, GamedayPanels, MenuItem};
use crate::live_game::GameState;
use crate::schedule::ScheduleState;
use mlb_api::client::{MLBApi, MLBApiBuilder};

use crossbeam_channel::{bounded, select, unbounded, Receiver, Sender};
use crossterm::event::Event;
use crossterm::{cursor, execute, terminal};
use lazy_static::lazy_static;
use tui::{backend::CrosstermBackend, Terminal};

const UPDATE_INTERVAL: u64 = 10; // seconds
lazy_static! {
    static ref CLIENT: MLBApi = MLBApiBuilder::default().build().unwrap();
    pub static ref REDRAW_REQUEST: (Sender<()>, Receiver<()>) = bounded(1);
    pub static ref SCHEDULE_CHANGE: (Sender<()>, Receiver<()>) = bounded(1);
}

fn main() -> Result<(), Box<dyn Error>> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();
    setup_terminal();

    let schedule_update = SCHEDULE_CHANGE.0.clone();
    let request_redraw = REDRAW_REQUEST.0.clone();
    let redraw_requested = REDRAW_REQUEST.1.clone();
    let ui_events = setup_ui_events();

    let app = Arc::new(Mutex::new(App {
        active_tab: MenuItem::Scoreboard,
        previous_state: MenuItem::Scoreboard,
        schedule: ScheduleState::from_schedule(&CLIENT.get_todays_schedule()),
        live_game: GameState::new(),
        debug_state: DebugState::Off,
        gameday: GamedayPanels::default(),
    }));

    // Network thread
    let network_app = app.clone();
    thread::spawn(move || {
        let app = network_app;
        let request_redraw = REDRAW_REQUEST.0.clone();
        let schedule = SCHEDULE_CHANGE.1.clone();

        // initial data load
        {
            let mut app = app.lock().unwrap();
            let game_id = app.schedule.get_selected_game();
            app.update_live_data(&CLIENT.get_live_data(game_id));
            let _ = request_redraw.try_send(());
        }

        loop {
            select! {
                // update only linescore when a different game is selected
                recv(schedule) -> _ => {
                    let mut app = app.lock().unwrap();
                    // TODO replace live_data with linescore endpoint for speed
                    let game_id = app.schedule.get_selected_game();
                    app.update_live_data(&CLIENT.get_live_data(game_id));
                    let _ = request_redraw.try_send(());
                }
                // do full update
                default(Duration::from_secs(UPDATE_INTERVAL)) => {
                    let mut app = app.lock().unwrap();
                    app.schedule.update(&CLIENT.get_todays_schedule());
                    let game_id = app.schedule.get_selected_game();
                    app.update_live_data(&CLIENT.get_live_data(game_id));
                    let _ = request_redraw.try_send(());
                }
            }
        }
    });

    loop {
        select! {
            recv(redraw_requested) -> _ => {
                let mut app = app.lock().unwrap();
                draw::draw(&mut terminal, &mut app);
            }

            recv(ui_events) -> message => {
                let mut app = app.lock().unwrap();
                match message {
                    Ok(Event::Key(key_event)) => {
                        event::handle_key_bindings(app.active_tab, key_event, &mut app, &request_redraw, &schedule_update);
                    }
                    Ok(Event::Resize(..)) => {
                        let _ = request_redraw.try_send(());
                    }
                    _ => {}
                }
            }

            // Default redraw on every duration
            default(Duration::from_millis(500)) => {
                let mut app = app.lock().unwrap();
                draw::draw(&mut terminal, &mut app);
            }
        }
    }
}

fn setup_terminal() {
    let mut stdout = io::stdout();

    execute!(stdout, cursor::Hide).unwrap();
    execute!(stdout, terminal::EnterAlternateScreen).unwrap();
    execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();

    terminal::enable_raw_mode().unwrap();
}

fn cleanup_terminal() {
    let mut stdout = io::stdout();

    execute!(stdout, cursor::MoveTo(0, 0)).unwrap();
    execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
    execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
    execute!(stdout, cursor::Show).unwrap();

    terminal::disable_raw_mode().unwrap();
}

fn setup_ui_events() -> Receiver<Event> {
    let (sender, receiver) = unbounded();
    std::thread::spawn(move || loop {
        sender.send(crossterm::event::read().unwrap()).unwrap();
    });
    receiver
}
