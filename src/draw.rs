use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use tui::{Frame, Terminal};

use crate::app::{App, DebugState, MenuItem};
use crate::debug::DebugInfo;
use crate::ui::at_bat::AtBatWidget;
use crate::ui::boxscore_stats::TeamBatterBoxscoreWidget;
use crate::ui::help::{HelpWidget, DOCS_LEN};
use crate::ui::layout::LayoutAreas;
use crate::ui::linescore::LineScoreWidget;
use crate::ui::matchup::MatchupWidget;
use crate::ui::plays::InningPlaysWidget;
use crate::ui::schedule::ScheduleWidget;
use crate::ui::standings::StandingsWidget;
use crate::ui::tabs::render_top_bar;

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
                MenuItem::Scoreboard => draw_scoreboard(f, main_layout.main, app),
                MenuItem::DatePicker => {
                    draw_scoreboard(f, main_layout.main, app);
                    draw_date_picker(f, main_layout.main, app);
                }
                MenuItem::Gameday => draw_gameday(f, main_layout.main, app),
                MenuItem::Stats => {
                    let gameday = Paragraph::new("stats").block(tempblock.clone());
                    f.render_widget(gameday, main_layout.main);
                }
                MenuItem::Standings => {
                    f.render_stateful_widget(
                        StandingsWidget {},
                        main_layout.main,
                        &mut app.standings,
                    );
                }
                MenuItem::Help => draw_help(f, f.size()),
            }
            if app.debug_state == DebugState::On {
                let mut dbi = DebugInfo::new();
                dbi.gather_info(f, &app);
                dbi.render(f, main_layout.main)
            }
        })
        .unwrap();
}

fn draw_border<B>(f: &mut Frame<B>, rect: Rect, color: Color)
where
    B: Backend,
{
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color));
    f.render_widget(block, rect);
}

fn draw_scoreboard<B>(f: &mut Frame<B>, rect: Rect, app: &mut App)
where
    B: Backend,
{
    let chunks = LayoutAreas::for_boxscore(rect);
    // add borders around the line score
    draw_border(f, chunks[0], Color::White);

    app.live_game.linescore.mini = false;
    f.render_stateful_widget(LineScoreWidget {}, chunks[0], &mut app.live_game.linescore);
    f.render_stateful_widget(ScheduleWidget {}, chunks[1], &mut app.schedule);
}

fn draw_date_picker<B>(f: &mut Frame<B>, rect: Rect, app: &mut App)
where
    B: Backend,
{
    let chunk = LayoutAreas::create_date_picker(rect);
    f.render_widget(Clear, chunk);

    let lines = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1), // top border
                Constraint::Length(1), // directions
                Constraint::Length(1), // input
            ]
            .as_ref(),
        )
        .split(chunk);

    let directions = Paragraph::new(" Press Enter to submit or Esc to cancel");
    f.render_widget(directions, lines[1]);

    let input = Paragraph::new(" > ");
    f.render_widget(input, lines[2]);

    let block = Block::default()
        .title("Enter a date in YYYY-MM-DD form")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue));
    f.render_widget(block, chunk);
}

fn draw_gameday<B>(f: &mut Frame<B>, rect: Rect, app: &mut App)
where
    B: Backend,
{
    let mut panels = LayoutAreas::generate_gameday_panels(&app.gameday, rect);

    // I want the panels to be displayed [Info, Heat, Box] from left to right. So pop off
    // available panels starting with Box. Since `generate_layouts` takes into account how many
    // panels are active, all the pops are guaranteed to unwrap.
    if app.gameday.boxscore {
        let p = panels.pop().unwrap();
        draw_border(f, p, Color::White);
        app.live_game.linescore.mini = true;
        f.render_stateful_widget(LineScoreWidget {}, p, &mut app.live_game.linescore);
        f.render_stateful_widget(
            TeamBatterBoxscoreWidget {
                active: app.boxscore_tab,
            },
            p,
            &mut app.live_game.boxscore,
        );
    }
    if app.gameday.at_bat {
        let p = panels.pop().unwrap();
        draw_border(f, p, Color::White);
        f.render_stateful_widget(AtBatWidget {}, p, &mut app.live_game.at_bat);
    }
    if app.gameday.info {
        let p = panels.pop().unwrap();
        draw_border(f, p, Color::White);
        f.render_stateful_widget(MatchupWidget {}, p, &mut app.live_game.matchup);
        f.render_stateful_widget(InningPlaysWidget {}, p, &mut app.live_game.plays);
    }
}

fn draw_help<B>(f: &mut Frame<B>, rect: Rect)
where
    B: Backend,
{
    f.render_widget(Clear, rect);

    // if the terminal is too small display a red border
    let mut color = Color::White;
    let min_height = DOCS_LEN as u16 + 3; // 3 for table header, top border, bottom border
    if rect.height < min_height || rect.width < 35 {
        color = Color::Red;
    }
    draw_border(f, rect, color);

    f.render_widget(HelpWidget {}, rect);
}
