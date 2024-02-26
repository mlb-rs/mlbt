use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Line, Span};
use tui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Tabs};
use tui::{Frame, Terminal};

use crate::app::{App, DebugState, MenuItem, HomeOrAway};
use crate::components::debug::DebugInfo;
use crate::ui::at_bat::AtBatWidget;
use crate::ui::boxscore::TeamBatterBoxscoreWidget;
use crate::ui::help::{HelpWidget, DOCS_LEN};
use crate::ui::layout::LayoutAreas;
use crate::ui::linescore::LineScoreWidget;
use crate::ui::matchup::MatchupWidget;
use crate::ui::plays::InningPlaysWidget;
use crate::ui::schedule::ScheduleWidget;
use crate::ui::standings::StandingsWidget;
use crate::ui::stats::{StatsWidget, STATS_OPTIONS_WIDTH};

static TABS: &[&str; 4] = &["Scoreboard", "Gameday", "Stats", "Standings"];

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
            main_layout.update(f.size(), app.settings.full_screen);

            if !app.settings.full_screen {
                draw_tabs(f, &main_layout.top_bar, app);
            }

            match app.state.active_tab {
                MenuItem::Scoreboard => draw_scoreboard(f, main_layout.main, app),
                MenuItem::DatePicker => {
                    match app.state.previous_tab {
                        MenuItem::Scoreboard => draw_scoreboard(f, main_layout.main, app),
                        MenuItem::Stats => draw_stats(f, main_layout.main, app),
                        MenuItem::Standings => draw_standings(f, main_layout.main, app),
                        _ => ()
                    }

                    draw_date_picker(f, main_layout.main, app);
                }
                MenuItem::Gameday => draw_gameday(f, main_layout.main, app),
                MenuItem::Stats => draw_stats(f, main_layout.main, app),
                MenuItem::Standings => draw_standings(f, main_layout.main, app),
                MenuItem::Help => draw_help(f, f.size()),
            }
            if app.state.debug_state == DebugState::On {
                let mut dbi = DebugInfo::new();
                dbi.gather_info(f, app);
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

fn draw_tabs<B>(f: &mut Frame<B>, top_bar: &[Rect], app: &App)
where
    B: Backend,
{
    let style = Style::default().fg(Color::White);
    let border_style = Style::default();
    let border_type = BorderType::Rounded;

    let titles = TABS
        .iter()
        .enumerate()
        .map(|(i, t)| {
            // underline the active tab
            if i == app.state.active_tab as usize {
                Line::from(Span::styled(
                    *t,
                    Style::default().add_modifier(Modifier::UNDERLINED),
                ))
            } else {
                Line::from(*t)
            }
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::BOTTOM | Borders::TOP)
                .border_type(border_type)
                .border_style(border_style),
        )
        .style(style);
    f.render_widget(tabs, top_bar[0]);

    let help = Paragraph::new("Help: ? ")
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::RIGHT | Borders::BOTTOM | Borders::TOP)
                .border_type(border_type)
                .border_style(border_style),
        )
        .style(style);
    f.render_widget(help, top_bar[1]);
}

fn draw_scoreboard<B>(f: &mut Frame<B>, rect: Rect, app: &mut App)
where
    B: Backend,
{
    // TODO calculate width based on table sizes
    let direction = match f.size().width {
        w if w < 125 => Direction::Vertical,
        _ => Direction::Horizontal,
    };
    let chunks = Layout::default()
        .direction(direction)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(rect);

    // display scores on left side
    f.render_stateful_widget(ScheduleWidget {}, chunks[0], &mut app.state.schedule);

    // display line score and box score on right
    draw_border(f, chunks[1], Color::White);
    draw_linescore_boxscore(f, chunks[1], app);
}

fn draw_linescore_boxscore<B>(f: &mut Frame<B>, rect: Rect, app: &mut App)
where
    B: Backend,
{
    let chunks = LayoutAreas::for_boxscore(rect);

    app.state.live_game.linescore.mini = true;
    f.render_stateful_widget(
        LineScoreWidget {
            active: app.state.boxscore_tab,
        },
        chunks[0],
        &mut app.state.live_game.linescore,
    );

    // Away Team Box Score
    f.render_stateful_widget(
        TeamBatterBoxscoreWidget {
            active: HomeOrAway::Away
        },
        chunks[1],
        &mut app.state.live_game.boxscore,
    );

    // Home Team Box Score
    f.render_stateful_widget(
        TeamBatterBoxscoreWidget {
            active: HomeOrAway::Home
        },
        chunks[2],
        &mut app.state.live_game.boxscore
    );
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

    let input = Paragraph::new(format!(" {}", app.state.date_input.text));
    f.render_widget(input, lines[2]);

    let border = match app.state.date_input.is_valid {
        true => Style::default().fg(Color::Blue),
        false => Style::default().fg(Color::Red),
    };
    let block = Block::default()
        .title("Enter a date (YYYY-MM-DD) or use arrow keys")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border);
    f.render_widget(block, chunk);

    // display cursor
    f.set_cursor(
        lines[2].x + app.state.date_input.text.len() as u16 + 1,
        lines[2].y,
    )
}

fn draw_gameday<B>(f: &mut Frame<B>, rect: Rect, app: &mut App)
where
    B: Backend,
{
    let mut panels = LayoutAreas::generate_gameday_panels(&app.state.gameday, rect);

    // I want the panels to be displayed [Info, Heat, Box] from left to right. So pop off
    // available panels starting with Box. Since `generate_layouts` takes into account how many
    // panels are active, all the pops are guaranteed to unwrap.
    if app.state.gameday.boxscore {
        let p = panels.pop().unwrap();
        draw_border(f, p, Color::White);
        draw_linescore_boxscore(f, p, app);
    }
    if app.state.gameday.at_bat {
        let p = panels.pop().unwrap();
        draw_border(f, p, Color::White);
        f.render_stateful_widget(AtBatWidget {}, p, &mut app.state.live_game.at_bat);
    }
    if app.state.gameday.info {
        let p = panels.pop().unwrap();
        draw_border(f, p, Color::White);
        f.render_stateful_widget(MatchupWidget {}, p, &mut app.state.live_game.matchup);
        f.render_stateful_widget(InningPlaysWidget {}, p, &mut app.state.live_game.plays);
    }
}

fn draw_stats<B>(f: &mut Frame<B>, rect: Rect, app: &mut App)
where
    B: Backend,
{
    // TODO by taking into account the width of the options pane I'm basically removing that amount
    // of space for columns. If I didn't, you could select columns that would be covered by the
    // options pane, but then when its disabled would become visible.
    let width = match app.state.stats.show_options {
        true => rect.width - STATS_OPTIONS_WIDTH,
        false => rect.width,
    };
    app.state.stats.trim_columns(width);
    f.render_stateful_widget(
        StatsWidget {
            show_options: app.state.stats.show_options,
        },
        rect,
        &mut app.state.stats,
    );
}

fn draw_standings<B>(f: &mut Frame<B>, rect: Rect, app: &mut App)
where
    B: Backend,
{
    f.render_stateful_widget(
        StandingsWidget {},
        rect,
        &mut app.state.standings,
    );
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
