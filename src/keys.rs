use crate::app::{App, DebugState, MenuItem};
use crate::cleanup_terminal;
use crate::components::stats::table::TeamOrPlayer;
use crate::state::messages::{NetworkRequest, RefreshableRequest};
use crate::state::settings_editor::SettingsFocus;
use crate::state::stats::ActivePane;
use crossterm::event::KeyCode::Char;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mlbt_api::client::StatGroup;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard, mpsc};

type AppGuard<'a> = MutexGuard<'a, App>;

pub async fn handle_key_bindings(
    key_event: KeyEvent,
    app: &Arc<Mutex<App>>,
    network_requests: &mpsc::Sender<RefreshableRequest>,
) {
    let mut guard = app.lock().await;
    match (guard.state.active_tab, key_event.code, key_event.modifiers) {
        // Ctrl+C always quits
        (_, Char('c'), KeyModifiers::CONTROL) => {
            cleanup_terminal();
            std::process::exit(0);
        }

        // Team page from standings
        (MenuItem::Standings, KeyCode::Esc, _) if guard.state.standings.has_team_page() => {
            guard.close_overlay();
        }
        (MenuItem::Standings, _, _) if guard.state.standings.has_team_page() => {
            let handled = handle_team_page_key(
                key_event,
                &mut guard.state.standings.team_page,
                network_requests,
            )
            .await;
            if !handled {
                handle_global_key(key_event, guard, network_requests).await;
            }
        }

        // Team page from stats
        (MenuItem::Stats, KeyCode::Esc, _) if guard.state.stats.has_team_page() => {
            guard.close_overlay();
        }
        (MenuItem::Stats, _, _) if guard.state.stats.has_team_page() => {
            let handled = handle_team_page_key(
                key_event,
                &mut guard.state.stats.team_page,
                network_requests,
            )
            .await;
            if !handled {
                handle_global_key(key_event, guard, network_requests).await;
            }
        }

        // Player profile handles its own keys; unhandled ones fall through to global bindings
        (MenuItem::Stats, KeyCode::Esc, _) if guard.state.stats.has_player_profile() => {
            guard.close_overlay();
        }
        (MenuItem::Stats, _, _) if guard.state.stats.has_player_profile() => {
            let date = guard.state.stats.date_selector.date;
            let handled = handle_profile_key(
                key_event,
                &mut guard.state.stats.player_profile,
                date,
                network_requests,
            )
            .await;
            if !handled {
                handle_global_key(key_event, guard, network_requests).await;
            }
        }

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

        (MenuItem::Scoreboard, Char('J') | KeyCode::Down, KeyModifiers::SHIFT) => {
            guard.state.box_score.scroll_down()
        }
        (MenuItem::Scoreboard, Char('j') | KeyCode::Down, _) => {
            guard.state.schedule.next();
            load_game_data(guard, network_requests, false).await;
        }
        (MenuItem::Scoreboard, Char('K') | KeyCode::Up, KeyModifiers::SHIFT) => {
            guard.state.box_score.scroll_up()
        }
        (MenuItem::Scoreboard, Char('k') | KeyCode::Up, _) => {
            guard.state.schedule.previous();
            load_game_data(guard, network_requests, false).await;
        }
        (MenuItem::Scoreboard, Char(':'), _) => guard.update_tab(MenuItem::DatePicker),
        (MenuItem::Scoreboard, Char('w'), _) => guard.state.schedule.toggle_win_probability(),
        (MenuItem::Scoreboard, KeyCode::Enter, _) => {
            guard.update_tab(MenuItem::Gameday);
            load_game_data(guard, network_requests, false).await;
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
            load_stats(guard, network_requests, false).await;
        }
        (MenuItem::Stats, Char('h'), _) => {
            guard.state.stats.stat_type.group = StatGroup::Hitting;
            load_stats(guard, network_requests, false).await;
        }
        (MenuItem::Stats, Char('l'), _) => {
            guard.state.stats.stat_type.team_player = TeamOrPlayer::Player;
            load_stats(guard, network_requests, false).await;
        }
        (MenuItem::Stats, Char('t'), _) => {
            guard.state.stats.stat_type.team_player = TeamOrPlayer::Team;
            load_stats(guard, network_requests, false).await;
        }
        (MenuItem::Stats, KeyCode::Enter, _) => {
            if guard.state.stats.active_pane == ActivePane::Options {
                guard.state.stats.toggle_stat();
            } else {
                open_stats_selection(guard, network_requests).await;
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
        (MenuItem::Standings, KeyCode::Enter, _) => load_team(guard, network_requests).await,
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

        (MenuItem::Help, _, _) => {
            let (handled, game_id_changed) = handle_help_key(key_event, &mut guard);
            if game_id_changed && guard.state.previous_tab == MenuItem::Scoreboard {
                load_game_data(guard, network_requests, false).await;
            } else if !handled {
                handle_global_key(key_event, guard, network_requests).await;
            }
        }

        _ => handle_global_key(key_event, guard, network_requests).await,
    }
}

async fn load_team(guard: AppGuard<'_>, network_requests: &mpsc::Sender<RefreshableRequest>) {
    let team_id = guard.state.standings.get_selected();
    let date = guard.state.standings.date_selector.date;
    drop(guard);

    let _ = network_requests
        .send(NetworkRequest::TeamPage { team_id, date }.into())
        .await;
}

async fn load_game_data(
    guard: AppGuard<'_>,
    network_requests: &mpsc::Sender<RefreshableRequest>,
    force: bool,
) {
    let game_id = guard.state.schedule.get_selected_game_opt();
    drop(guard);

    if let Some(game_id) = game_id {
        let _ = network_requests
            .send(RefreshableRequest::new(
                NetworkRequest::GameData { game_id },
                force,
            ))
            .await;
    }
}

async fn load_stats(
    guard: AppGuard<'_>,
    network_requests: &mpsc::Sender<RefreshableRequest>,
    force: bool,
) {
    let date = guard.state.stats.date_selector.date;
    let stat_type = guard.state.stats.stat_type;
    drop(guard);

    let _ = network_requests
        .send(RefreshableRequest::new(
            NetworkRequest::Stats { date, stat_type },
            force,
        ))
        .await;
}

async fn open_stats_selection(
    guard: AppGuard<'_>,
    network_requests: &mpsc::Sender<RefreshableRequest>,
) {
    if let Some(request) = guard.state.stats.open_selected_request() {
        drop(guard);
        let _ = network_requests.send(request.into()).await;
    }
}

async fn load_standings(
    guard: AppGuard<'_>,
    network_requests: &mpsc::Sender<RefreshableRequest>,
    force: bool,
) {
    let date = guard.state.standings.date_selector.date;
    drop(guard);

    let _ = network_requests
        .send(RefreshableRequest::new(
            NetworkRequest::Standings { date },
            force,
        ))
        .await;
}

async fn load_scoreboard(
    guard: AppGuard<'_>,
    network_requests: &mpsc::Sender<RefreshableRequest>,
    force: bool,
) {
    let date = guard.state.schedule.date_selector.date;
    let game_id = guard.state.schedule.get_selected_game_opt();
    drop(guard);

    let _ = network_requests
        .send(RefreshableRequest::new(
            NetworkRequest::Schedule { date },
            force,
        ))
        .await;

    if let Some(game_id) = game_id {
        let _ = network_requests
            .send(RefreshableRequest::new(
                NetworkRequest::GameData { game_id },
                force,
            ))
            .await;
    }
}

async fn handle_date_change(
    guard: AppGuard<'_>,
    network_requests: &mpsc::Sender<RefreshableRequest>,
) {
    match guard.state.active_tab {
        MenuItem::Scoreboard => load_scoreboard(guard, network_requests, false).await,
        MenuItem::Standings => load_standings(guard, network_requests, false).await,
        MenuItem::Stats => load_stats(guard, network_requests, false).await,
        _ => {}
    }
}

/// Handle player profile key bindings shared across Stats tab and team page contexts.
/// Returns true if the key was consumed.
async fn handle_profile_key(
    key_event: KeyEvent,
    profile: &mut Option<crate::state::player_profile::PlayerProfileState>,
    date: chrono::NaiveDate,
    network_requests: &mpsc::Sender<RefreshableRequest>,
) -> bool {
    let Some(p) = profile.as_mut() else {
        return false;
    };
    match (key_event.code, key_event.modifiers) {
        (Char('s'), _) => {
            let req = p.game_type_toggle_request(date);
            let _ = network_requests.send(req.into()).await;
        }
        (Char('J'), _) | (KeyCode::Down, KeyModifiers::SHIFT) => p.page_down(),
        (Char('K'), _) | (KeyCode::Up, KeyModifiers::SHIFT) => p.page_up(),
        (Char('j') | KeyCode::Down, _) => p.scroll_down(),
        (Char('k') | KeyCode::Up, _) => p.scroll_up(),
        _ => return false,
    }
    true
}

async fn handle_global_key(
    key_event: KeyEvent,
    mut guard: AppGuard<'_>,
    network_requests: &mpsc::Sender<RefreshableRequest>,
) {
    match (key_event.code, key_event.modifiers) {
        (Char('q'), _) => {
            cleanup_terminal();
            std::process::exit(0);
        }
        (Char('f'), m) if !m.contains(KeyModifiers::CONTROL) => guard.toggle_full_screen(),
        (Char('1'), _) => {
            let force = guard.state.active_tab == MenuItem::Scoreboard;
            guard.update_tab(MenuItem::Scoreboard);
            guard.state.gameday.live(); // reset at bat selection
            load_scoreboard(guard, network_requests, force).await;
        }
        (Char('2'), _) => {
            let force = guard.state.active_tab == MenuItem::Gameday;
            guard.update_tab(MenuItem::Gameday);
            load_game_data(guard, network_requests, force).await;
        }
        (Char('3'), _) => {
            let force = guard.state.active_tab == MenuItem::Stats;
            guard.update_tab(MenuItem::Stats);
            if !guard.state.stats.has_player_profile() {
                load_stats(guard, network_requests, force).await;
            }
        }
        (Char('4'), _) => {
            let force = guard.state.active_tab == MenuItem::Standings;
            guard.update_tab(MenuItem::Standings);
            load_standings(guard, network_requests, force).await;
        }
        (Char('?'), _) => guard.update_tab(MenuItem::Help),
        (Char('d'), _) => guard.toggle_debug(),
        (Char('"'), _) => {
            if guard.state.debug_state == DebugState::On {
                guard.toggle_show_logs();
            }
        }
        _ => {}
    }
}

async fn handle_team_page_key(
    key_event: KeyEvent,
    team_page: &mut Option<crate::state::team_page::TeamPageState>,
    network_requests: &mpsc::Sender<RefreshableRequest>,
) -> bool {
    let Some(tp) = team_page.as_mut() else {
        return false;
    };

    if tp.has_player_profile() {
        return handle_profile_key(key_event, &mut tp.player_profile, tp.date, network_requests)
            .await;
    }

    match (key_event.code, key_event.modifiers) {
        (KeyCode::Right | KeyCode::Tab, _) => tp.next_section(),
        (KeyCode::Left, _) => tp.previous_section(),
        (Char('J'), _) | (KeyCode::Down, KeyModifiers::SHIFT) => tp.page_down(),
        (Char('K'), _) | (KeyCode::Up, KeyModifiers::SHIFT) => tp.page_up(),
        (Char('j') | KeyCode::Down, _) => tp.next(),
        (Char('k') | KeyCode::Up, _) => tp.previous(),
        (Char('c'), _) => tp.toggle_calendar(),
        (Char('r'), _) => {
            let req = tp.roster_toggle_request();
            let _ = network_requests.send(req.into()).await;
        }
        (KeyCode::Enter, _) => {
            if let Some(req) = tp.player_profile_request() {
                let _ = network_requests.send(req.into()).await;
            }
        }
        _ => return false,
    }
    true
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

/// Handle keys on the Help page.
///
/// Returns `(handled, game_id_changed)`, where:
///
/// - `handled`: `true` if the key was consumed, `false` to fall through to the global handler.
/// - `game_id_changed`: whether committing a picker changed the selected schedule game id.
fn handle_help_key(key_event: KeyEvent, guard: &mut AppGuard<'_>) -> (bool, bool) {
    // picker overlay intercepts everything
    if guard.state.settings_editor.picker.is_some() {
        let mut game_id_changed = false;
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Esc, _) => guard.state.settings_editor.close_picker(),
            (KeyCode::Enter, _) => game_id_changed = guard.commit_settings_picker(),
            (Char('j') | KeyCode::Down, _) => guard.state.settings_editor.picker_next(),
            (Char('k') | KeyCode::Up, _) => guard.state.settings_editor.picker_previous(),
            // letter keys (except j/k which stay as nav) jump to the next option starting with that
            // letter. cycles on repeated presses.
            (Char(c), _) if c.is_ascii_alphabetic() => {
                guard.state.settings_editor.picker_jump_to_char(c);
            }
            _ => {}
        }
        return (true, game_id_changed);
    }

    if key_event.code == KeyCode::Tab {
        guard.state.settings_editor.toggle_focus();
        return (true, false);
    }

    if let (Char('"'), _) = (key_event.code, key_event.modifiers) {
        guard.toggle_show_logs();
        return (true, false);
    }

    let handled = match guard.state.settings_editor.focus {
        SettingsFocus::Settings => match (key_event.code, key_event.modifiers) {
            (Char('j') | KeyCode::Down, _) => {
                guard.state.settings_editor.next_field();
                true
            }
            (Char('k') | KeyCode::Up, _) => {
                guard.state.settings_editor.previous_field();
                true
            }
            (KeyCode::Enter, _) => {
                let field = guard.state.settings_editor.selected_field;
                let cursor = field.current_index(&guard.settings);
                guard.state.settings_editor.open_picker(cursor);
                true
            }
            (KeyCode::Esc, _) => {
                guard.exit_help();
                true
            }
            _ => false,
        },
        SettingsFocus::Docs => match (key_event.code, key_event.modifiers) {
            (Char('J'), _) | (KeyCode::Down, KeyModifiers::SHIFT) => {
                guard.state.help.page_down();
                true
            }
            (Char('K'), _) | (KeyCode::Up, KeyModifiers::SHIFT) => {
                guard.state.help.page_up();
                true
            }
            (Char('j') | KeyCode::Down, _) => {
                guard.state.help.next();
                true
            }
            (Char('k') | KeyCode::Up, _) => {
                guard.state.help.previous();
                true
            }
            (KeyCode::Esc, _) => {
                guard.exit_help();
                true
            }
            _ => false,
        },
    };
    (handled, false)
}
