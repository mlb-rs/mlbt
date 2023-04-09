mod app;
mod at_bat;
mod banner;
mod boxscore;
mod constants;
mod debug;
mod draw;
mod event;
mod linescore;
mod live_game;
mod matchup;
mod pitches;
mod plays;
mod schedule;
mod standings;
mod stats;
mod strikezone;
mod ui;
mod util;

use std::io::Stdout;
use std::sync::Arc;
use std::time::Duration;
use std::{io, panic};

use crate::app::{App, DateInput, DebugState, GamedayPanels, HomeOrAway, MenuItem};
use crate::live_game::GameState;
use crate::schedule::ScheduleState;
use crate::standings::StandingsState;
use crate::stats::{StatsState, TeamOrPlayer};
use crossbeam_channel::{bounded, select, unbounded, Receiver, Sender};
use crossterm::event::Event;
use crossterm::{cursor, execute, terminal};
use mlb_api::client::{MLBApi, MLBApiBuilder};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tui::{backend::CrosstermBackend, Terminal};

const UPDATE_INTERVAL: u64 = 10; // seconds
static CLIENT: Lazy<MLBApi> = Lazy::new(|| MLBApiBuilder::default().build().unwrap());
pub static REDRAW_REQUEST: Lazy<(Sender<()>, Receiver<()>)> = Lazy::new(|| bounded(1));
pub static UPDATE_REQUEST: Lazy<(Sender<MenuItem>, Receiver<MenuItem>)> = Lazy::new(|| bounded(1));

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    better_panic::install();

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    setup_panic_hook();
    setup_terminal();

    let ui_events = setup_ui_events();

    let app = Arc::new(Mutex::new(App {
        active_tab: MenuItem::Scoreboard,
        previous_tab: MenuItem::Scoreboard,
        full_screen: false,
        schedule: ScheduleState::from_schedule(&CLIENT.get_todays_schedule().await),
        date_input: DateInput::default(),
        standings: StandingsState::default(),
        stats: StatsState::default(),
        live_game: GameState::new(),
        debug_state: DebugState::Off,
        gameday: GamedayPanels::default(),
        boxscore_tab: HomeOrAway::Home,
    }));

    // Network thread
    tokio::spawn({
        let app = app.clone();
        async move {
            network_thread(app).await;
        }
    });

    ui_thread(&mut terminal, ui_events, app.clone()).await;

    Ok(())
}

async fn ui_thread(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ui_events: Receiver<Event>,
    app: Arc<Mutex<App>>,
) {
    let schedule_update = UPDATE_REQUEST.0.clone();
    let request_redraw = REDRAW_REQUEST.0.clone();
    let redraw_requested = REDRAW_REQUEST.1.clone();

    loop {
        select! {
            recv(redraw_requested) -> _ => {
                let mut app = app.lock().await;
                draw::draw(terminal, &mut app);
            }

            recv(ui_events) -> message => {
                let mut app = app.lock().await;
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
                let mut app = app.lock().await;
                draw::draw(terminal, &mut app);
            }
        }
    }
}

async fn network_thread(app: Arc<Mutex<App>>) {
    let request_redraw = REDRAW_REQUEST.0.clone();
    let update_received = UPDATE_REQUEST.1.clone();

    // initial data load
    {
        let mut app = app.lock().await;
        let game_id = app.schedule.get_selected_game();
        app.update_live_data(&CLIENT.get_live_data(game_id).await);
        let _ = request_redraw.try_send(());
    }

    loop {
        select! {
            recv(update_received) -> message => {
                let mut app = app.lock().await;
                match message {
                    // update linescore only when a different game is selected
                    Ok(MenuItem::Scoreboard) => {
                        // TODO replace live_data with linescore endpoint for speed
                        let game_id = app.schedule.get_selected_game();
                        app.update_live_data(&CLIENT.get_live_data(game_id).await);
                    }
                    // update schedule and linescore when a new date is picked
                    Ok(MenuItem::DatePicker) => {
                        let (schedule, game) = tokio::join!(
                            CLIENT.get_schedule_date(app.schedule.date),
                            CLIENT.get_live_data(app.schedule.get_selected_game())
                        );
                        app.schedule.update(&schedule);
                        app.update_live_data(&game);
                    }
                    // update standings only when tab is switched to
                    Ok(MenuItem::Standings) => {
                        app.standings.update(&CLIENT.get_standings().await);
                    }
                    // update stats only when tab is switched to, team/player is changed, or
                    // pitching/hitting is changed
                    Ok(MenuItem::Stats) => {
                        let response = match app.stats.stat_type.team_player {
                            TeamOrPlayer::Team => CLIENT.get_team_stats(app.stats.stat_type.group.clone()).await,
                            TeamOrPlayer::Player => CLIENT.get_player_stats(app.stats.stat_type.group.clone()).await,
                        };
                        app.stats.update(&response);
                    }
                    _ => {}
                }
                let _ = request_redraw.try_send(());
            }
            // do full update
            default(Duration::from_secs(UPDATE_INTERVAL)) => {
                let mut app = app.lock().await;
                match app.active_tab {
                    MenuItem::Scoreboard => {
                        let (schedule, game) = tokio::join!(
                            CLIENT.get_schedule_date(app.schedule.date),
                            CLIENT.get_live_data(app.schedule.get_selected_game())
                        );
                        app.schedule.update(&schedule);
                        app.update_live_data(&game);
                    },
                    MenuItem::Gameday => {
                        let game_id = app.schedule.get_selected_game();
                        app.update_live_data(&CLIENT.get_live_data(game_id).await);
                    },
                    MenuItem::Standings => {
                        // Don't update the standings every 10 seconds, only on tab switch
                    },
                    MenuItem::Stats => {},
                    MenuItem::DatePicker => {},
                    MenuItem::Help => {},
                }
                let _ = request_redraw.try_send(());
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

fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        cleanup_terminal();
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));
}
