use crate::app::{App, DebugState, MenuItem};
use crate::components::stats::TeamOrPlayer;
use crate::state::app_state::HomeOrAway;
use crate::{NetworkRequest, cleanup_terminal};
use crossterm::event::KeyCode::Char;
use crossterm::event::{KeyCode, KeyEvent};
use mlb_api::client::StatGroup;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard, mpsc};

type AppGuard<'a> = MutexGuard<'a, App>;

pub async fn handle_key_bindings(
    key_event: KeyEvent,
    app: &Arc<Mutex<App>>,
    network_requests: &mpsc::Sender<NetworkRequest>,
) {
    let mut guard = app.lock().await;

    match (guard.state.active_tab, key_event.code) {
        (_, Char('q')) => {
            cleanup_terminal();
            std::process::exit(0);
        }

        // needs to be before the tab switches to capture number inputs
        (MenuItem::DatePicker, Char(c)) => {
            guard.state.date_input.is_valid = true; // reset status
            guard.state.date_input.text.push(c);
        }

        (_, Char('f')) => guard.toggle_full_screen(),
        (_, Char('1')) => {
            guard.update_tab(MenuItem::Scoreboard);
            guard.state.gameday.live(); // reset at bat selection
            load_scoreboard(guard, network_requests).await;
        }
        (_, Char('2')) => {
            guard.update_tab(MenuItem::Gameday);
            load_game_data(guard, network_requests).await;
        }
        (_, Char('3')) => {
            guard.update_tab(MenuItem::Stats);
            load_stats(guard, network_requests).await;
        }
        (_, Char('4')) => {
            guard.update_tab(MenuItem::Standings);
            load_standings(guard, network_requests).await;
        }

        (MenuItem::Scoreboard, Char('j')) => {
            guard.state.schedule.next();
            load_game_data(guard, network_requests).await;
        }
        (MenuItem::Scoreboard, Char('k')) => {
            guard.state.schedule.previous();
            load_game_data(guard, network_requests).await;
        }
        (MenuItem::Scoreboard, Char(':')) => guard.update_tab(MenuItem::DatePicker),
        (MenuItem::Scoreboard, Char('w')) => guard.state.schedule.toggle_win_probability(),
        (MenuItem::Scoreboard, KeyCode::Enter) => {
            guard.update_tab(MenuItem::Gameday);
            load_game_data(guard, network_requests).await;
        }

        (MenuItem::DatePicker, KeyCode::Enter) => {
            if guard.try_update_date_from_input().is_ok() {
                let previous_tab = guard.state.previous_tab;
                guard.update_tab(previous_tab);
                handle_date_change(guard, network_requests).await;
            }
        }
        (MenuItem::DatePicker, KeyCode::Right) => {
            guard.move_date_selector_by_arrow(true);
        }
        (MenuItem::DatePicker, KeyCode::Left) => {
            guard.move_date_selector_by_arrow(false);
        }
        (MenuItem::DatePicker, KeyCode::Esc) => {
            guard.state.date_input.text.clear();
            let previous_tab = guard.state.previous_tab;
            guard.update_tab(previous_tab);
        }
        (MenuItem::DatePicker, KeyCode::Backspace) => {
            guard.state.date_input.text.pop();
        }

        (MenuItem::Stats, Char('j')) => guard.state.stats.next(),
        (MenuItem::Stats, Char('k')) => guard.state.stats.previous(),
        (MenuItem::Stats, Char('o')) => {
            guard.state.stats.show_options = !guard.state.stats.show_options
        }
        (MenuItem::Stats, Char('p')) => {
            guard.state.stats.stat_type.group = StatGroup::Pitching;
            load_stats(guard, network_requests).await;
        }
        (MenuItem::Stats, Char('h')) => {
            guard.state.stats.stat_type.group = StatGroup::Hitting;
            load_stats(guard, network_requests).await;
        }
        (MenuItem::Stats, Char('l')) => {
            guard.state.stats.stat_type.team_player = TeamOrPlayer::Player;
            load_stats(guard, network_requests).await;
        }
        (MenuItem::Stats, Char('t')) => {
            guard.state.stats.stat_type.team_player = TeamOrPlayer::Team;
            load_stats(guard, network_requests).await;
        }
        (MenuItem::Stats, KeyCode::Enter) => guard.state.stats.toggle_stat(),
        (MenuItem::Stats, Char('s')) => guard.state.stats.store_sort_column(),
        (MenuItem::Stats, Char(':')) => guard.update_tab(MenuItem::DatePicker),

        (MenuItem::Standings, Char('j')) => guard.state.standings.next(),
        (MenuItem::Standings, Char('k')) => guard.state.standings.previous(),
        (MenuItem::Standings, KeyCode::Enter) => {
            let _team_id = guard.state.standings.get_selected();
            // println!("team id: {:?}", team_id);
            // TODO show team info panel
        }
        (MenuItem::Standings, Char(':')) => guard.update_tab(MenuItem::DatePicker),

        (MenuItem::Gameday, Char('i')) => guard.state.gameday.toggle_info(),
        (MenuItem::Gameday, Char('p')) => guard.state.gameday.toggle_at_bat(),
        (MenuItem::Gameday, Char('b')) => guard.state.gameday.toggle_boxscore(),
        (MenuItem::Gameday, Char('w')) => guard.state.gameday.toggle_win_probability(),
        (MenuItem::Gameday, Char('j')) => guard.state.gameday.previous_at_bat(),
        (MenuItem::Gameday, Char('k')) => guard.state.gameday.next_at_bat(),
        (MenuItem::Gameday, Char('l')) => guard.state.gameday.live(),
        (MenuItem::Gameday, Char('s')) => guard.state.gameday.start(),

        (MenuItem::Gameday, Char('h')) => guard.state.boxscore_tab = HomeOrAway::Home,
        (MenuItem::Gameday, Char('a')) => guard.state.boxscore_tab = HomeOrAway::Away,
        (MenuItem::Scoreboard, Char('h')) => guard.state.boxscore_tab = HomeOrAway::Home,
        (MenuItem::Scoreboard, Char('a')) => guard.state.boxscore_tab = HomeOrAway::Away,

        (_, Char('?')) => guard.update_tab(MenuItem::Help),
        (MenuItem::Help, KeyCode::Esc) => guard.exit_help(),
        (_, Char('d')) => guard.toggle_debug(),
        (MenuItem::Help, Char('"')) => guard.toggle_show_logs(),
        (_, Char('"')) => {
            if guard.state.debug_state == DebugState::On {
                guard.toggle_show_logs();
            }
        }

        _ => {}
    }
}

async fn load_game_data(guard: AppGuard<'_>, network_requests: &mpsc::Sender<NetworkRequest>) {
    let game_id = guard.state.schedule.get_selected_game_opt();
    drop(guard);

    if let Some(game_id) = game_id {
        let _ = network_requests
            .send(NetworkRequest::GameData { game_id })
            .await;
    }
}

async fn load_stats(guard: AppGuard<'_>, network_requests: &mpsc::Sender<NetworkRequest>) {
    let date = guard.state.stats.date_selector.date;
    let stat_type = guard.state.stats.stat_type;
    drop(guard);

    let _ = network_requests
        .send(NetworkRequest::Stats { date, stat_type })
        .await;
}

async fn load_standings(guard: AppGuard<'_>, network_requests: &mpsc::Sender<NetworkRequest>) {
    let date = guard.state.standings.date_selector.date;
    drop(guard);

    let _ = network_requests
        .send(NetworkRequest::Standings { date })
        .await;
}

async fn load_scoreboard(guard: AppGuard<'_>, network_requests: &mpsc::Sender<NetworkRequest>) {
    let date = guard.state.schedule.date_selector.date;
    let game_id = guard.state.schedule.get_selected_game_opt();
    drop(guard);

    let _ = network_requests
        .send(NetworkRequest::Schedule { date })
        .await;

    if let Some(game_id) = game_id {
        let _ = network_requests
            .send(NetworkRequest::GameData { game_id })
            .await;
    }
}

async fn handle_date_change(guard: AppGuard<'_>, network_requests: &mpsc::Sender<NetworkRequest>) {
    match guard.state.active_tab {
        MenuItem::Scoreboard => load_scoreboard(guard, network_requests).await,
        MenuItem::Standings => load_standings(guard, network_requests).await,
        MenuItem::Stats => load_stats(guard, network_requests).await,
        _ => {}
    }
}
