mod app;
mod components;
mod config;
mod draw;
mod keys;
mod state;
mod ui;

use crate::app::App;
use crate::state::messages::{NetworkRequest, NetworkResponse, UiEvent};
use crate::state::network::{LoadingState, NetworkWorker};
use crate::state::refresher::PeriodicRefresher;
use crossterm::event::{self as crossterm_event, Event};
use crossterm::{cursor, execute, terminal};
use log::error;
use std::io::Stdout;
use std::sync::Arc;
use std::{io, panic};
use tokio::sync::{Mutex, mpsc};
use tui::{Terminal, backend::CrosstermBackend};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    better_panic::install();

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;

    setup_panic_hook();
    setup_terminal();

    // initialize logging
    tui_logger::init_logger(log::LevelFilter::Error)?;
    tui_logger::set_default_level(log::LevelFilter::Error);

    let app = Arc::new(Mutex::new(App::new()));

    let (ui_event_tx, ui_event_rx) = mpsc::channel::<UiEvent>(100);
    let (network_req_tx, network_req_rx) = mpsc::channel::<NetworkRequest>(100);
    let (network_resp_tx, network_resp_rx) = mpsc::channel::<NetworkResponse>(100);

    // input handler thread
    let input_handler = tokio::spawn(input_handler_task(ui_event_tx.clone()));

    // network thread
    let network_worker = NetworkWorker::new(network_req_rx, network_resp_tx);
    let network_task = tokio::spawn(network_worker.run());

    // periodic update thread
    let periodic_updater = PeriodicRefresher::new(network_req_tx.clone());
    let periodic_task = tokio::spawn(periodic_updater.run(app.clone()));

    // send initial app started event
    let _ = ui_event_tx.send(UiEvent::AppStarted).await;

    // run the main UI loop
    main_ui_loop(terminal, app, ui_event_rx, network_req_tx, network_resp_rx).await;

    input_handler.abort();
    network_task.abort();
    periodic_task.abort();

    Ok(())
}

async fn main_ui_loop(
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
    app: Arc<Mutex<App>>,
    mut ui_events: mpsc::Receiver<UiEvent>,
    network_requests: mpsc::Sender<NetworkRequest>,
    mut network_responses: mpsc::Receiver<NetworkResponse>,
) {
    let mut loading = LoadingState::default();

    loop {
        tokio::select! {
            Some(ui_event) = ui_events.recv() => {
                let should_redraw = handle_ui_event(ui_event, &app, &network_requests).await;
                if should_redraw && !loading.is_loading {
                    let mut app_guard = app.lock().await;
                    draw::draw(&mut terminal, &mut app_guard, loading);
                }
            }

            Some(response) = network_responses.recv() => {
                let should_redraw = handle_network_response(response, &app, &network_requests, &mut loading).await;
                if should_redraw {
                    let mut app_guard = app.lock().await;
                    draw::draw(&mut terminal, &mut app_guard, loading);
                }
            }
        }
    }
}

async fn handle_ui_event(
    ui_event: UiEvent,
    app: &Arc<Mutex<App>>,
    network_requests: &mpsc::Sender<NetworkRequest>,
) -> bool {
    match ui_event {
        UiEvent::AppStarted => {
            let date = {
                let guard = app.lock().await;
                guard.state.schedule.date_selector.date
            };
            let _ = network_requests
                .send(NetworkRequest::Schedule { date })
                .await;
            true // Redraw immediately to show loading state
        }
        UiEvent::KeyPressed(key_event) => {
            keys::handle_key_bindings(key_event, app, network_requests).await;
            true // Redraw after key handling
        }
        UiEvent::Resize => true, // Redraw on resize
    }
}

async fn handle_network_response(
    response: NetworkResponse,
    app: &Arc<Mutex<App>>,
    network_requests: &mpsc::Sender<NetworkRequest>,
    loading: &mut LoadingState,
) -> bool {
    match response {
        NetworkResponse::LoadingStateChanged { loading_state } => {
            *loading = loading_state;
            // Always redraw for loading changes
            return true;
        }
        NetworkResponse::ScheduleLoaded { schedule } => {
            let game_id_to_load = {
                let mut guard = app.lock().await;
                guard.update_schedule(&schedule)
            };

            if let Some(game_id) = game_id_to_load {
                let _ = network_requests
                    .send(NetworkRequest::GameData { game_id })
                    .await;
            }
        }
        NetworkResponse::GameDataLoaded {
            game,
            win_probability,
        } => {
            let mut guard = app.lock().await;
            guard.update_live_data(&game, &win_probability);
        }
        NetworkResponse::StandingsLoaded { standings } => {
            let mut guard = app.lock().await;
            guard.state.standings.update(&standings);
        }
        NetworkResponse::StatsLoaded { stats } => {
            let mut guard = app.lock().await;
            guard.state.stats.update(&stats);
        }
        NetworkResponse::Error { message } => {
            error!("Network error: {message}");
        }
    }
    // Only redraw if not loading
    !loading.is_loading
}

async fn input_handler_task(ui_events: mpsc::Sender<UiEvent>) {
    loop {
        if let Ok(event) = crossterm_event::read() {
            let ui_event = match event {
                Event::Key(key_event) => Some(UiEvent::KeyPressed(key_event)),
                Event::Resize(_, _) => Some(UiEvent::Resize),
                Event::Mouse(_) => None, // Ignore mouse events for now
                _ => None,
            };

            if let Some(ui_event) = ui_event
                && ui_events.send(ui_event).await.is_err()
            {
                break; // Channel closed, exit task
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

fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        cleanup_terminal();
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));
}
