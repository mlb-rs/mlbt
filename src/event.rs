use crate::app::MenuItem;
use crate::{app, cleanup_terminal};
use crossbeam_channel::Sender;
use termion::event::Key;
use termion::event::Key::Char;

pub fn handle_key_bindings(
    mode: MenuItem,
    key_event: Key,
    mut app: &mut app::App,
    request_redraw: &Sender<()>,
) {
    match (mode, key_event) {
        (_, Char('q')) => {
            cleanup_terminal();
            std::process::exit(0);
        }
        (_, Char('1')) => app.update_tab(MenuItem::Scoreboard),
        (_, Char('2')) => app.update_tab(MenuItem::Gameday),
        (_, Char('3')) => app.update_tab(MenuItem::Stats),
        (_, Char('4')) => app.update_tab(MenuItem::Standings),

        // (_, Char('j')) => app.schedule.next(),
        // (_, Char('k')) => app.schedule.previous(),
        (_, Char('?')) => app.exit_help(),
        (_, Key::Esc) => app.update_tab(MenuItem::Help),
        (_, Char('d')) => app.toggle_debug(),

        (MenuItem::Gameday, Char('i')) => {
            // toggle info
        }
        (MenuItem::Gameday, Char('p')) => {
            // toggle pitches
        }
        (MenuItem::Gameday, Char('b')) => {
            // toggle box score
        }
        (MenuItem::Gameday, Char('h')) => {
            // home box score
        }
        (MenuItem::Gameday, Char('a')) => {
            // away box score
        }
        _ => {}
    }
    let _ = request_redraw.try_send(());
}
