use crate::app::{HomeOrAway, MenuItem};
use crate::components::stats::TeamOrPlayer;
use crate::{app, cleanup_terminal};
use crossbeam_channel::Sender;
use crossterm::event::KeyCode::Char;
use crossterm::event::{KeyCode, KeyEvent};
use mlb_api::client::StatGroup;

pub fn handle_key_bindings(
    key_event: KeyEvent,
    app: &mut app::App,
    request_redraw: &Sender<()>,
    selective_update: &Sender<MenuItem>,
) {
    match (app.state.active_tab, key_event.code) {
        (_, Char('q')) => {
            cleanup_terminal();
            std::process::exit(0);
        }

        // needs to be before the tab switches to capture number inputs
        (MenuItem::DatePicker, Char(c)) => {
            app.state.date_input.is_valid = true; // reset status
            app.state.date_input.text.push(c);
        }

        (_, Char('f')) => app.toggle_full_screen(),
        (_, Char('1')) => app.update_tab(MenuItem::Scoreboard),
        (_, Char('2')) => app.update_tab(MenuItem::Gameday),
        (_, Char('3')) => {
            app.update_tab(MenuItem::Stats);
            let _ = selective_update.try_send(MenuItem::Stats);
        }
        (_, Char('4')) => {
            app.update_tab(MenuItem::Standings);
            let _ = selective_update.try_send(MenuItem::Standings);
        }

        (MenuItem::Scoreboard, Char('j')) => {
            app.state.schedule.next();
            let _ = selective_update.try_send(MenuItem::Scoreboard);
        }
        (MenuItem::Scoreboard, Char('k')) => {
            app.state.schedule.previous();
            let _ = selective_update.try_send(MenuItem::Scoreboard);
        }
        (MenuItem::Scoreboard, Char(':')) => app.update_tab(MenuItem::DatePicker),

        (MenuItem::DatePicker, KeyCode::Enter) => {
            let date: String = app.state.date_input.text.drain(..).collect();
            match app.state.previous_tab {
                MenuItem::Scoreboard => {
                    if app.state.schedule.set_date_from_input(date).is_ok() {
                        app.state.date_input.is_valid = true;
                        app.update_tab(MenuItem::Scoreboard);
                        let _ = selective_update.try_send(MenuItem::DatePicker);
                    } else {
                        app.state.date_input.is_valid = false;
                    }
                }
                MenuItem::Standings => {
                    if app.state.standings.set_date_from_input(date).is_ok() {
                        app.state.date_input.is_valid = true;
                        app.update_tab(MenuItem::Standings);
                        let _ = selective_update.try_send(MenuItem::DatePicker);
                    } else {
                        app.state.date_input.is_valid = false;
                    }
                }
                _ => (),
            }
        }
        (MenuItem::DatePicker, KeyCode::Right) => {
            let date = match app.state.previous_tab {
                MenuItem::Scoreboard => Some(app.state.schedule.set_date_with_arrows(true)),
                MenuItem::Standings => Some(app.state.standings.set_date_with_arrows(true)),
                _ => None,
            };
            app.state.date_input.text.clear();
            if let Some(date) = date {
                app.state.date_input.text.push_str(&date.to_string());
            }
        }
        (MenuItem::DatePicker, KeyCode::Left) => {
            let date = app.state.schedule.set_date_with_arrows(false);
            app.state.date_input.text.clear();
            app.state.date_input.text.push_str(&date.to_string());
        }
        (MenuItem::DatePicker, KeyCode::Esc) => {
            app.state.date_input.text.clear();
            match app.state.previous_tab {
                MenuItem::Scoreboard => app.update_tab(MenuItem::Scoreboard),
                MenuItem::Standings => app.update_tab(MenuItem::Standings),
                _ => (),
            }
        }
        (MenuItem::DatePicker, KeyCode::Backspace) => {
            app.state.date_input.text.pop();
        }

        (MenuItem::Stats, Char('j')) => app.state.stats.next(),
        (MenuItem::Stats, Char('k')) => app.state.stats.previous(),
        (MenuItem::Stats, Char('o')) => {
            app.state.stats.show_options = !app.state.stats.show_options
        }
        (MenuItem::Stats, Char('p')) => {
            app.state.stats.stat_type.group = StatGroup::Pitching;
            let _ = selective_update.try_send(MenuItem::Stats);
        }
        (MenuItem::Stats, Char('h')) => {
            app.state.stats.stat_type.group = StatGroup::Hitting;
            let _ = selective_update.try_send(MenuItem::Stats);
        }
        (MenuItem::Stats, Char('l')) => {
            app.state.stats.stat_type.team_player = TeamOrPlayer::Player;
            let _ = selective_update.try_send(MenuItem::Stats);
        }
        (MenuItem::Stats, Char('t')) => {
            app.state.stats.stat_type.team_player = TeamOrPlayer::Team;
            let _ = selective_update.try_send(MenuItem::Stats);
        }
        (MenuItem::Stats, KeyCode::Enter) => app.state.stats.toggle_stat(),
        (MenuItem::Stats, Char('s')) => app.state.stats.store_sort_column(),

        (MenuItem::Standings, Char('j')) => app.state.standings.next(),
        (MenuItem::Standings, Char('k')) => app.state.standings.previous(),
        (MenuItem::Standings, KeyCode::Enter) => {
            let _team_id = app.state.standings.get_selected();
            // println!("team id: {:?}", team_id);
            // TODO show team info panel
        }
        (MenuItem::Standings, Char(':')) => app.update_tab(MenuItem::DatePicker),

        (MenuItem::Gameday, Char('i')) => app.state.gameday.info = !app.state.gameday.info,
        (MenuItem::Gameday, Char('p')) => app.state.gameday.at_bat = !app.state.gameday.at_bat,
        (MenuItem::Gameday, Char('b')) => app.state.gameday.boxscore = !app.state.gameday.boxscore,

        // TODO use bitflags to enable (MenuItem::Gameday | MenuItem::Scoreboard)?
        (MenuItem::Gameday, Char('h')) => app.state.boxscore_tab = HomeOrAway::Home,
        (MenuItem::Gameday, Char('a')) => app.state.boxscore_tab = HomeOrAway::Away,
        (MenuItem::Scoreboard, Char('h')) => app.state.boxscore_tab = HomeOrAway::Home,
        (MenuItem::Scoreboard, Char('a')) => app.state.boxscore_tab = HomeOrAway::Away,

        (_, Char('?')) => app.update_tab(MenuItem::Help),
        (MenuItem::Help, KeyCode::Esc) => app.exit_help(),
        (_, Char('d')) => app.toggle_debug(),

        _ => {}
    }
    let _ = request_redraw.try_send(());
}
