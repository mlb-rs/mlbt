use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Tabs, Wrap};
use tui::{Frame, Terminal};

use crate::app::{App, DebugState, MenuItem};
use crate::debug::DebugInfo;
use crate::gameday::{AtBatPanel, BoxPanel, Gameday, GamedayPanel, InfoPanel};
use crate::linescore::LineScore;
use crate::ui::at_bat::AtBatWidget;
use crate::ui::boxscore_stats::TeamBatterBoxscoreWidget;
use crate::ui::help::render_help;
use crate::ui::layout::LayoutAreas;
use crate::ui::linescore::LineScoreWidget;
use crate::ui::matchup::MatchupWidget;
use crate::ui::plays::InningPlaysWidget;
use crate::ui::schedule::ScheduleWidget;
use crate::ui::tabs::render_top_bar;
use mlb_api::live::Linescore;

pub fn draw<B>(terminal: &mut Terminal<B>, app: &mut App)
where
    B: Backend,
{
    let current_size = terminal.size().unwrap_or_default();
    let mut main_layout = LayoutAreas::new(current_size);

    if current_size.width <= 10 || current_size.height <= 10 {
        return;
    }

    terminal
        .draw(|f| {
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

                    app.live_game.linescore.mini = false;
                    f.render_stateful_widget(
                        LineScoreWidget {},
                        chunks[0],
                        &mut app.live_game.linescore,
                    );
                }
                MenuItem::Gameday => {
                    draw_gameday(f, main_layout.main, app);
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

fn draw_gameday<B>(f: &mut Frame<B>, rect: Rect, app: &mut App)
where
    B: Backend,
{
    let mut panels = app.gameday.generate_layouts(rect);

    // I want the panels to be displayed [Info, Heat, Box] from left to right. So pop off
    // available panels starting with Box. Since `generate_layouts` takes into account how many
    // panels are active, all the pops are guaranteed to unwrap.
    if app.gameday.boxscore.active {
        let p = panels.pop().unwrap();
        BoxPanel::draw_border(f, p);
        app.live_game.linescore.mini = true;
        f.render_stateful_widget(LineScoreWidget {}, p, &mut app.live_game.linescore);
        f.render_stateful_widget(
            TeamBatterBoxscoreWidget {},
            p,
            &mut app.gameday.boxscore.stats,
        );
    }
    if app.gameday.at_bat.active {
        let p = panels.pop().unwrap();
        AtBatPanel::draw_border(f, p);
        f.render_stateful_widget(AtBatWidget {}, p, &mut app.live_game.at_bat);
    }
    if app.gameday.info.active {
        let p = panels.pop().unwrap();
        InfoPanel::draw_border(f, p);
        f.render_stateful_widget(MatchupWidget {}, p, &mut app.live_game.matchup);
        f.render_stateful_widget(InningPlaysWidget {}, p, &mut app.live_game.plays);
    }
}
