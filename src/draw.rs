use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Tabs, Wrap};
use tui::{Frame, Terminal};

use crate::app::{App, DebugState, MenuItem};
use crate::boxscore::BoxScore;
use crate::debug::DebugInfo;
use crate::ui::help::render_help;
use crate::ui::layout::LayoutAreas;
use crate::ui::tabs::render_top_bar;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    let current_size = terminal.size().unwrap_or_default();
    let mut main_layout = LayoutAreas::new(current_size);

    if current_size.width <= 10 || current_size.height <= 10 {
        return;
    }

    terminal
        .draw(|mut f| {
            main_layout.update(f.size());
            render_top_bar(f, &main_layout.top_bar);

            let tempblock = Block::default().borders(Borders::ALL);
            match app.active_tab {
                MenuItem::Scoreboard => {
                    // Create block for rendering boxscore and schedule
                    let layout = main_layout.for_boxscore();

                    // Hit the API to update the schedule
                    // app.schedule.update(&mlb.get_todays_schedule());
                    // app.schedule.render(f, layout[1]);

                    // // Hit the API to get live game data TODO add error handling
                    // let game_id = app.schedule.get_selected_game();
                    // let live_game = mlb.get_live_data(game_id);
                    //
                    // // temp
                    // let block = Block::default()
                    //     .borders(Borders::ALL)
                    //     .border_type(BorderType::Rounded);
                    // f.render_widget(block, layout[0]);
                    // let boxscore = BoxScore::from_live_data(&live_game);
                    // boxscore.render(f, layout[0]);
                }
                MenuItem::Gameday => {
                    // let game_id = app.schedule.get_selected_game();
                    // let live_game = mlb.get_live_data(game_id);

                    // app.gameday.load_live_data(&live_game);
                    // app.gameday.render(f, main_layout.main, &app);
                }
                MenuItem::Stats => {
                    let gameday = Paragraph::new("stats").block(tempblock.clone());
                    f.render_widget(gameday, main_layout.main);
                }
                MenuItem::Standings => {
                    let gameday = Paragraph::new("standings").block(tempblock.clone());
                    f.render_widget(gameday, main_layout.main);
                }
                MenuItem::Help => render_help(f),
            }
            if app.debug_state == DebugState::On {
                let mut dbi = DebugInfo::new();
                dbi.gather_info(f, &app);
                dbi.render(f, main_layout.main)
            }
        })
        .unwrap();
}
