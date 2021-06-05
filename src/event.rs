use crate::app::{BoxscoreTab, MenuItem};
use crate::{app, cleanup_terminal};
use crossbeam_channel::Sender;
use crossterm::event::KeyCode::Char;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_bindings(
    mode: MenuItem,
    key_event: KeyEvent,
    mut app: &mut app::App,
    request_redraw: &Sender<()>,
    schedule_update: &Sender<()>,
) {
    match (mode, key_event.code) {
        (_, Char('q')) => {
            cleanup_terminal();
            std::process::exit(0);
        }
        (_, Char('1')) => app.update_tab(MenuItem::Scoreboard),
        (_, Char('2')) => app.update_tab(MenuItem::Gameday),
        (_, Char('3')) => app.update_tab(MenuItem::Stats),
        (_, Char('4')) => app.update_tab(MenuItem::Standings),

        (MenuItem::Scoreboard, Char('j')) => {
            app.schedule.next();
            let _ = schedule_update.try_send(());
        }
        (MenuItem::Scoreboard, Char('k')) => {
            app.schedule.previous();
            let _ = schedule_update.try_send(());
        }

        (_, Char('?')) => app.update_tab(MenuItem::Help),
        (_, KeyCode::Esc) => app.exit_help(),
        (_, Char('d')) => app.toggle_debug(),

        (MenuItem::Gameday, Char('i')) => app.gameday.info = !app.gameday.info,
        (MenuItem::Gameday, Char('p')) => app.gameday.at_bat = !app.gameday.at_bat,
        (MenuItem::Gameday, Char('b')) => app.gameday.boxscore = !app.gameday.boxscore,
        (MenuItem::Gameday, Char('h')) => app.boxscore_tab = BoxscoreTab::Home,
        (MenuItem::Gameday, Char('a')) => app.boxscore_tab = BoxscoreTab::Away,
        _ => {}
    }
    let _ = request_redraw.try_send(());
}
