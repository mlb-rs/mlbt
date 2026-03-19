use crate::app::{App, DebugState, MenuItem};
use crate::components::stats::table::TeamOrPlayer;
use crate::state::stats::ActivePane;
use crate::{NetworkRequest, cleanup_terminal};
use crossterm::event::KeyCode::Char;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mlbt_api::client::StatGroup;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard, mpsc};

type AppGuard<'a> = MutexGuard<'a, App>;

pub async fn handle_key_bindings(
    key_event: KeyEvent,
    app: &Arc<Mutex<App>>,
    network_requests: &mpsc::Sender<NetworkRequest>,
) {
    let mut guard = app.lock().await;
    match (guard.state.active_tab, key_event.code, key_event.modifiers) {
        // Ctrl+C always quits, even during search
        (_, Char('c'), KeyModifiers::CONTROL) => {
            cleanup_terminal();
            std::process::exit(0);
        }

        // Escape closes player profile first
        (MenuItem::Stats, KeyCode::Esc, _) if guard.state.stats.has_player_profile() => {
            guard.state.stats.close_player_profile();
        }
        // When player profile is open, swallow all other Stats keys
        (MenuItem::Stats, _, _) if guard.state.stats.has_player_profile() => {}

        // in search mode capture all keys
        (MenuItem::Stats, _, _) if guard.state.stats.search.is_open => {
            handle_search_key(key_event, &mut guard);
        }

        // regular quit after checking search mode
        (_, Char('q'), _) => {
            cleanup_terminal();
            std::process::exit(0);
        }

        // needs to be before the tab switches to capture number inputs
        (MenuItem::DatePicker, Char(c), _) => {
            guard.state.date_input.is_valid = true; // reset status
            guard.state.date_input.text.push(c);
        }

        // Ctrl+F opens search in Stats tab, needs to be before `f` handler
        (MenuItem::Stats, Char('f'), KeyModifiers::CONTROL) => {
            guard.state.stats.open_search();
        }

        (_, Char('f'), _) => guard.toggle_full_screen(),
        (_, Char('1'), _) => {
            guard.update_tab(MenuItem::Scoreboard);
            guard.state.gameday.live(); // reset at bat selection
            load_scoreboard(guard, network_requests).await;
        }
        (_, Char('2'), _) => {
            guard.update_tab(MenuItem::Gameday);
            load_game_data(guard, network_requests).await;
        }
        (_, Char('3'), _) => {
            guard.update_tab(MenuItem::Stats);
            load_stats(guard, network_requests).await;
        }
        (_, Char('4'), _) => {
            guard.update_tab(MenuItem::Standings);
            load_standings(guard, network_requests).await;
        }

        (MenuItem::Scoreboard, Char('J') | KeyCode::Down, KeyModifiers::SHIFT) => {
            guard.state.box_score.scroll_down()
        }
        (MenuItem::Scoreboard, Char('j') | KeyCode::Down, _) => {
            guard.state.schedule.next();
            load_game_data(guard, network_requests).await;
        }
        (MenuItem::Scoreboard, Char('K') | KeyCode::Up, KeyModifiers::SHIFT) => {
            guard.state.box_score.scroll_up()
        }
        (MenuItem::Scoreboard, Char('k') | KeyCode::Up, _) => {
            guard.state.schedule.previous();
            load_game_data(guard, network_requests).await;
        }
        (MenuItem::Scoreboard, Char(':'), _) => guard.update_tab(MenuItem::DatePicker),
        (MenuItem::Scoreboard, Char('w'), _) => guard.state.schedule.toggle_win_probability(),
        (MenuItem::Scoreboard, KeyCode::Enter, _) => {
            guard.update_tab(MenuItem::Gameday);
            load_game_data(guard, network_requests).await;
        }

        (MenuItem::DatePicker, KeyCode::Enter, _) => {
            if guard.try_update_date_from_input().is_ok() {
                let previous_tab = guard.state.previous_tab;
                guard.update_tab(previous_tab);
                handle_date_change(guard, network_requests).await;
            }
        }
        (MenuItem::DatePicker, KeyCode::Right, _) => {
            guard.move_date_selector_by_arrow(true);
        }
        (MenuItem::DatePicker, KeyCode::Left, _) => {
            guard.move_date_selector_by_arrow(false);
        }
        (MenuItem::DatePicker, KeyCode::Esc, _) => {
            guard.state.date_input.text.clear();
            let previous_tab = guard.state.previous_tab;
            guard.update_tab(previous_tab);
        }
        (MenuItem::DatePicker, KeyCode::Backspace, _) => {
            guard.state.date_input.text.pop();
        }

        (MenuItem::Stats, Char('J') | KeyCode::Down, KeyModifiers::SHIFT) => {
            guard.state.stats.page_down()
        }
        (MenuItem::Stats, Char('K') | KeyCode::Up, KeyModifiers::SHIFT) => {
            guard.state.stats.page_up()
        }
        (MenuItem::Stats, KeyCode::PageDown, _) => guard.state.stats.page_down(),
        (MenuItem::Stats, KeyCode::PageUp, _) => guard.state.stats.page_up(),
        (MenuItem::Stats, Char('j') | KeyCode::Down, _) => guard.state.stats.next(),
        (MenuItem::Stats, Char('k') | KeyCode::Up, _) => guard.state.stats.previous(),
        (MenuItem::Stats, Char('o'), _) => guard.state.stats.toggle_options(),
        (MenuItem::Stats, Char('p'), _) => {
            guard.state.stats.stat_type.group = StatGroup::Pitching;
            load_stats(guard, network_requests).await;
        }
        (MenuItem::Stats, Char('h'), _) => {
            guard.state.stats.stat_type.group = StatGroup::Hitting;
            load_stats(guard, network_requests).await;
        }
        (MenuItem::Stats, Char('l'), _) => {
            guard.state.stats.stat_type.team_player = TeamOrPlayer::Player;
            load_stats(guard, network_requests).await;
        }
        (MenuItem::Stats, Char('t'), _) => {
            guard.state.stats.stat_type.team_player = TeamOrPlayer::Team;
            load_stats(guard, network_requests).await;
        }
        (MenuItem::Stats, KeyCode::Enter, _) => {
            if guard.state.stats.active_pane == ActivePane::Options {
                guard.state.stats.toggle_stat();
            } else {
                load_player_profile(guard, network_requests).await;
            }
        }
        (MenuItem::Stats, Char('s'), _) => guard.state.stats.store_sort_column(),
        (MenuItem::Stats, KeyCode::Left | KeyCode::Right | KeyCode::Tab, _) => {
            guard.state.stats.switch_pane()
        }
        (MenuItem::Stats, Char(':'), _) => guard.update_tab(MenuItem::DatePicker),

        (MenuItem::Standings, Char('j') | KeyCode::Down, _) => guard.state.standings.next(),
        (MenuItem::Standings, Char('k') | KeyCode::Up, _) => guard.state.standings.previous(),
        (MenuItem::Standings, Char('l'), _) => guard.state.standings.toggle_view_mode(),
        (MenuItem::Standings, KeyCode::Enter, _) => {
            let _team_id = guard.state.standings.get_selected();
            // println!("team id: {:?}", team_id);
            // TODO show team info panel
        }
        (MenuItem::Standings, Char(':'), _) => guard.update_tab(MenuItem::DatePicker),

        (MenuItem::Gameday, Char('i'), _) => guard.state.gameday.toggle_info(),
        (MenuItem::Gameday, Char('p'), _) => guard.state.gameday.toggle_at_bat(),
        (MenuItem::Gameday, Char('b'), _) => guard.state.gameday.toggle_boxscore(),
        (MenuItem::Gameday, Char('w'), _) => guard.state.gameday.toggle_win_probability(),
        (MenuItem::Gameday, Char('J') | KeyCode::Down, KeyModifiers::SHIFT) => {
            guard.state.box_score.scroll_down()
        }
        (MenuItem::Gameday, Char('K') | KeyCode::Up, KeyModifiers::SHIFT) => {
            guard.state.box_score.scroll_up()
        }
        (MenuItem::Gameday, Char('j') | KeyCode::Down, _) => guard.state.gameday.previous_at_bat(),
        (MenuItem::Gameday, Char('k') | KeyCode::Up, _) => guard.state.gameday.next_at_bat(),
        (MenuItem::Gameday, Char('l'), _) => guard.state.gameday.live(),
        (MenuItem::Gameday, Char('s'), _) => guard.state.gameday.start(),

        (MenuItem::Gameday, Char('h'), _) => guard.state.box_score.set_home_active(),
        (MenuItem::Gameday, Char('a'), _) => guard.state.box_score.set_away_active(),
        (MenuItem::Scoreboard, Char('h'), _) => guard.state.box_score.set_home_active(),
        (MenuItem::Scoreboard, Char('a'), _) => guard.state.box_score.set_away_active(),

        (_, Char('?'), _) => guard.update_tab(MenuItem::Help),
        (MenuItem::Help, Char('j') | KeyCode::Down, _) => guard.state.help.next(),
        (MenuItem::Help, Char('k') | KeyCode::Up, _) => guard.state.help.previous(),
        (MenuItem::Help, KeyCode::Esc, _) => guard.exit_help(),
        (_, Char('d'), _) => guard.toggle_debug(),
        (MenuItem::Help, Char('"'), _) => guard.toggle_show_logs(),
        (_, Char('"'), _) => {
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

async fn load_player_profile(guard: AppGuard<'_>, network_requests: &mpsc::Sender<NetworkRequest>) {
    if let Some((player_id, group, date)) = guard.state.stats.player_profile_request() {
        drop(guard);

        let _ = network_requests
            .send(NetworkRequest::PlayerProfile {
                player_id,
                group,
                date,
            })
            .await;
    }
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

fn handle_search_key(key_event: KeyEvent, guard: &mut AppGuard<'_>) {
    match (key_event.code, key_event.modifiers) {
        (Char(c), m) if !m.contains(KeyModifiers::CONTROL) => {
            guard.state.stats.search.handle_char(c);
            guard.state.stats.update_search_matches();
            guard.state.stats.reset_data_selection();
        }
        (KeyCode::Backspace, _) => {
            guard.state.stats.search.handle_backspace();
            guard.state.stats.update_search_matches();
            guard.state.stats.reset_data_selection();
        }
        (KeyCode::Esc, _) => {
            guard.state.stats.cancel_search();
        }
        (KeyCode::Enter, _) => {
            guard.state.stats.submit_search();
        }
        (KeyCode::Down, _) => guard.state.stats.next(),
        (KeyCode::Up, _) => guard.state.stats.previous(),
        _ => {} // swallow all other keys
    }
}
