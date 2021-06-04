use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Tabs, Wrap};
use tui::{Frame, Terminal};

use crate::app::{App, DebugState, MenuItem};
use crate::debug::DebugInfo;
use crate::linescore::LineScore;
use crate::ui::help::render_help;
use crate::ui::layout::LayoutAreas;
use crate::ui::linescore::LineScoreWidget;
use crate::ui::schedule::ScheduleWidget;
use crate::ui::tabs::render_top_bar;
use mlb_api::live::Linescore;

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
                    let chunks = LayoutAreas::for_boxscore(main_layout.main);

                    f.render_stateful_widget(ScheduleWidget {}, chunks[1], &mut app.schedule);

                    // add borders around the line score
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded);
                    f.render_widget(block, chunks[0]);

                    f.render_stateful_widget(
                        LineScoreWidget {},
                        chunks[0],
                        &mut app.live_game.linescore,
                    );
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
