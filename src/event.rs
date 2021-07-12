use crate::app::{HomeOrAway, MenuItem};
use crate::{app, cleanup_terminal};
use crossbeam_channel::Sender;
use crossterm::event::KeyCode::Char;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_bindings(
    mode: MenuItem,
    key_event: KeyEvent,
    mut app: &mut app::App,
    request_redraw: &Sender<()>,
    selective_update: &Sender<MenuItem>,
) {
    match (mode, key_event.code) {
        (_, Char('q')) => {
            cleanup_terminal();
            std::process::exit(0);
        }

        // needs to be before the tab switches to capture number inputs
        (MenuItem::DatePicker, Char(c)) => {
            app.date_input.is_valid = true; // reset status
            app.date_input.text.push(c);
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
            app.schedule.next();
            let _ = selective_update.try_send(MenuItem::Scoreboard);
        }
        (MenuItem::Scoreboard, Char('k')) => {
            app.schedule.previous();
            let _ = selective_update.try_send(MenuItem::Scoreboard);
        }
        (MenuItem::Scoreboard, Char(':')) => app.update_tab(MenuItem::DatePicker),

        (MenuItem::DatePicker, KeyCode::Enter) => {
            let date: String = app.date_input.text.drain(..).collect();
            if app.schedule.set_date_from_input(date).is_ok() {
                app.date_input.is_valid = true;
                app.update_tab(MenuItem::Scoreboard);
                let _ = selective_update.try_send(MenuItem::DatePicker);
            } else {
                app.date_input.is_valid = false;
            }
        }
        (MenuItem::DatePicker, KeyCode::Esc) => {
            app.date_input.text.clear();
            app.update_tab(MenuItem::Scoreboard)
        }
        (MenuItem::DatePicker, KeyCode::Backspace) => {
            app.date_input.text.pop();
        }

        (MenuItem::Stats, Char('j')) => app.stats.next(),
        (MenuItem::Stats, Char('k')) => app.stats.previous(),
        (MenuItem::Stats, Char('o')) => app.stats.stats_options = !app.stats.stats_options,
        (MenuItem::Stats, KeyCode::Enter) => app.stats.toggle_stat(),

        (MenuItem::Standings, Char('j')) => app.standings.next(),
        (MenuItem::Standings, Char('k')) => app.standings.previous(),
        (MenuItem::Standings, KeyCode::Enter) => {
            let team_id = app.standings.get_selected();
            println!("team id: {:?}", team_id);
            // TODO
        }

        (MenuItem::Gameday, Char('i')) => app.gameday.info = !app.gameday.info,
        (MenuItem::Gameday, Char('p')) => app.gameday.at_bat = !app.gameday.at_bat,
        (MenuItem::Gameday, Char('b')) => app.gameday.boxscore = !app.gameday.boxscore,

        // TODO use bitflags to enable (MenuItem::Gameday | MenuItem::Scoreboard)?
        (MenuItem::Gameday, Char('h')) => app.boxscore_tab = HomeOrAway::Home,
        (MenuItem::Gameday, Char('a')) => app.boxscore_tab = HomeOrAway::Away,
        (MenuItem::Scoreboard, Char('h')) => app.boxscore_tab = HomeOrAway::Home,
        (MenuItem::Scoreboard, Char('a')) => app.boxscore_tab = HomeOrAway::Away,

        (_, Char('?')) => app.update_tab(MenuItem::Help),
        (MenuItem::Help, KeyCode::Esc) => app.exit_help(),
        (_, Char('d')) => app.toggle_debug(),

        _ => {}
    }
    let _ = request_redraw.try_send(());
}
