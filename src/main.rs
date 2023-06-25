mod app;
mod components;
mod config;
mod draw;
mod event;
mod ui;

use std::io::Stdout;
use std::sync::Arc;
use std::time::Duration;
use std::{io, panic};

use crate::app::{App, MenuItem};
use crate::components::schedule::ScheduleState;
use crate::components::stats::TeamOrPlayer;
use crossbeam_channel::{bounded, select, Receiver};
use crossterm::event::Event;
use crossterm::{cursor, execute, terminal};
use tokio::sync::Mutex;
use tui::{backend::CrosstermBackend, Terminal};

const UPDATE_INTERVAL_SECONDS: u64 = 10;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    better_panic::install();

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    let app = App::new();
    let app = Arc::new(Mutex::new(app));

    setup_panic_hook();
    setup_terminal();

    // network thread
    tokio::spawn({
        let app = app.clone();
        async move {
            network_thread(app).await;
        }
    });

    let ui_events = setup_ui_events();
    ui_thread(&mut terminal, ui_events, app.clone()).await;

    Ok(())
}

async fn ui_thread(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ui_events: Receiver<Event>,
    app: Arc<Mutex<App>>,
) {
    let (schedule_update, request_redraw, redraw_requested) = {
        let app = app.lock().await;
        let schedule_update = app.update_channel.0.clone();
        let request_redraw = app.redraw_channel.0.clone();
        let redraw_requested = app.redraw_channel.1.clone();

        (schedule_update, request_redraw, redraw_requested)
    };

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
                        event::handle_key_bindings(key_event, &mut app, &request_redraw, &schedule_update);
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
    let (request_redraw, update_received) = {
        let mut app = app.lock().await;
        let request_redraw = app.redraw_channel.0.clone();
        let update_received = app.update_channel.1.clone();

        // initial data load
        let schedule = app.client.get_todays_schedule().await;
        app.state.schedule = ScheduleState::from_schedule(&app.settings, &schedule);

        let game = app
            .client
            .get_live_data(app.state.schedule.get_selected_game())
            .await;
        app.update_live_data(&game);
        let _ = request_redraw.try_send(());

        (request_redraw, update_received)
    };

    loop {
        select! {
            recv(update_received) -> message => {
                let mut app = app.lock().await;
                match message {
                    // update linescore only when a different game is selected
                    Ok(MenuItem::Scoreboard) => {
                        // TODO replace live_data with linescore endpoint for speed
                        let game = app.client.get_live_data(app.state.schedule.get_selected_game()).await;
                        app.update_live_data(&game);
                    }
                    // update schedule and linescore when a new date is picked
                    Ok(MenuItem::DatePicker) => {
                        let schedule = app.client.get_schedule_date(app.state.schedule.date).await;
                        app.update_schedule(&schedule);
                        // run sequentially to get the correct selected game id
                        let game_id = app.state.schedule.get_selected_game();
                        let game = app.client.get_live_data(game_id).await;
                        app.update_live_data(&game);
                    }
                    // update standings only when tab is switched to
                    Ok(MenuItem::Standings) => {
                        let standings = app.client.get_standings().await;
                        app.state.standings.update(&standings);
                    }
                    // update stats only when tab is switched to, team/player is changed, or
                    // pitching/hitting is changed
                    Ok(MenuItem::Stats) => {
                        let response = match app.state.stats.stat_type.team_player {
                            TeamOrPlayer::Team => app.client.get_team_stats(app.state.stats.stat_type.group.clone()).await,
                            TeamOrPlayer::Player => app.client.get_player_stats(app.state.stats.stat_type.group.clone()).await,
                        };
                        app.state.stats.update(&response);
                    }
                    _ => {}
                }
                let _ = request_redraw.try_send(());
            }
            // do full update
            default(Duration::from_secs(UPDATE_INTERVAL_SECONDS)) => {
                let mut app = app.lock().await;
                match app.state.active_tab {
                    MenuItem::Scoreboard => {
                        // run concurrently since game id is already correct
                        let (schedule, game) = tokio::join!(
                            app.client.get_schedule_date(app.state.schedule.date),
                            app.client.get_live_data(app.state.schedule.get_selected_game())
                        );
                        app.update_schedule(&schedule);
                        app.update_live_data(&game);
                    },
                    MenuItem::Gameday => {
                        let game = app.client.get_live_data(app.state.schedule.get_selected_game()).await;
                        app.update_live_data(&game);
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
    let (sender, receiver) = bounded(100);
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
