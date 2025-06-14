use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Line;
use tui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Tabs};
use tui::{Frame, Terminal};

use crate::app::{App, DebugState, MenuItem};
use crate::components::debug::DebugInfo;
use crate::state::network::LoadingState;
use crate::ui::boxscore::TeamBatterBoxscoreWidget;
use crate::ui::date_selector::DateSelectorWidget;
use crate::ui::gameday::gameday_widget::GamedayWidget;
use crate::ui::gameday::win_probability::WinProbabilityWidget;
use crate::ui::help::{DOCS, HelpWidget};
use crate::ui::layout::LayoutAreas;
use crate::ui::linescore::LineScoreWidget;
use crate::ui::schedule::ScheduleWidget;
use crate::ui::standings::StandingsWidget;
use crate::ui::stats::{STATS_OPTIONS_WIDTH, StatsWidget};

static TABS: &[&str; 4] = &["Scoreboard", "Gameday", "Stats", "Standings"];

pub fn draw<B>(terminal: &mut Terminal<B>, app: &mut App, is_loading: LoadingState)
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
            main_layout.update(f.area(), app.settings.full_screen);

            if !app.settings.full_screen {
                draw_tabs(f, &main_layout.top_bar, app, is_loading);
            }

            match app.state.active_tab {
                MenuItem::Scoreboard => draw_scoreboard(f, main_layout.main, app),
                MenuItem::DatePicker => {
                    match app.state.previous_tab {
                        MenuItem::Scoreboard => draw_scoreboard(f, main_layout.main, app),
                        MenuItem::Standings => draw_standings(f, main_layout.main, app),
                        MenuItem::Stats => draw_stats(f, main_layout.main, app),
                        _ => (),
                    }
                    draw_date_picker(f, main_layout.main, app);
                }
                MenuItem::Gameday => draw_gameday(f, main_layout.main, app),
                MenuItem::Stats => draw_stats(f, main_layout.main, app),
                MenuItem::Standings => draw_standings(f, main_layout.main, app),
                MenuItem::Help => draw_help(f, f.area()),
            }
            if app.state.debug_state == DebugState::On {
                let mut dbi = DebugInfo::new();
                dbi.gather_info(f, app);
                dbi.render(f, main_layout.main)
            }
        })
        .unwrap();
}

pub fn default_border<'a>(color: Color) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color))
}

fn draw_border(f: &mut Frame, rect: Rect, color: Color) {
    let block = default_border(color);
    f.render_widget(block, rect);
}

fn draw_tabs(f: &mut Frame, top_bar: &[Rect], app: &App, loading: LoadingState) {
    let style = Style::default().fg(Color::White);
    let border_style = Style::default();
    let border_type = BorderType::Rounded;

    let titles: Vec<Line> = TABS.iter().map(|t| Line::from(*t)).collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::BOTTOM | Borders::TOP)
                .border_type(border_type)
                .border_style(border_style),
        )
        .highlight_style(
            // underline the active tab
            Style::default().add_modifier(Modifier::UNDERLINED),
        )
        .select(app.state.active_tab as usize)
        .style(style);
    f.render_widget(tabs, top_bar[0]);

    // display animated spinner if there are API requests in progress
    let text = format!("{} Help: ? ", loading.spinner_char);
    let help = Paragraph::new(text)
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

fn draw_scoreboard(f: &mut Frame, rect: Rect, app: &mut App) {
    // TODO calculate width based on table sizes
    let direction = match f.area().width {
        w if w < 125 => Direction::Vertical,
        _ => Direction::Horizontal,
    };
    let chunks = Layout::default()
        .direction(direction)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(rect);

    // display scores on left side
    f.render_stateful_widget(
        ScheduleWidget {
            tz_abbreviation: app.settings.timezone_abbreviation.clone(),
        },
        chunks[0],
        &mut app.state.schedule,
    );
    if app.state.schedule.show_win_probability {
        draw_win_probability(f, chunks[0], app);
    }

    // display line score and box score on right
    draw_border(f, chunks[1], Color::White);
    draw_linescore_boxscore(f, chunks[1], app);
}

fn draw_linescore_boxscore(f: &mut Frame, rect: Rect, app: &App) {
    let chunks = LayoutAreas::for_boxscore(rect);

    f.render_widget(
        LineScoreWidget {
            active: app.state.boxscore_tab,
            linescore: &app.state.gameday.game.linescore,
        },
        chunks[0],
    );
    f.render_widget(
        TeamBatterBoxscoreWidget {
            active: app.state.boxscore_tab,
            boxscore: &app.state.gameday.game.boxscore,
        },
        chunks[1],
    );
}

fn draw_date_picker(f: &mut Frame, rect: Rect, app: &mut App) {
    let chunk = LayoutAreas::create_date_picker(rect);
    f.render_stateful_widget(DateSelectorWidget {}, chunk, &mut app.state.date_input);

    // display cursor
    f.set_cursor_position((
        chunk.x + app.state.date_input.text.len() as u16 + 1, // +1 for border
        chunk.y + 2,                                          // +2 for border and instructions
    ))
}

fn draw_gameday(f: &mut Frame, rect: Rect, app: &App) {
    f.render_widget(
        GamedayWidget {
            active: app.state.boxscore_tab,
            state: &app.state.gameday,
        },
        rect,
    );
}

fn draw_stats(f: &mut Frame, rect: Rect, app: &mut App) {
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

fn draw_standings(f: &mut Frame, rect: Rect, app: &mut App) {
    f.render_stateful_widget(StandingsWidget {}, rect, &mut app.state.standings);
}

fn draw_win_probability(f: &mut Frame, rect: Rect, app: &mut App) {
    // only render if it doesn't overlap the schedule
    let minimum_size =
        WinProbabilityWidget::get_min_table_height() + app.state.schedule.schedule.len() + 2; // +2 for borders 
    if rect.height > minimum_size as u16 {
        f.render_widget(
            WinProbabilityWidget {
                game: &app.state.gameday.game,
                selected_at_bat: app.state.gameday.selected_at_bat(),
                active_tab: MenuItem::Scoreboard,
            },
            rect,
        );
    }
}

fn draw_help(f: &mut Frame, rect: Rect) {
    f.render_widget(Clear, rect);

    // if the terminal is too small display a red border
    let mut color = Color::White;
    let min_height = DOCS.len() as u16 + 3; // 3 for table header, top border, bottom border
    if rect.height < min_height || rect.width < 35 {
        color = Color::Red;
    }
    draw_border(f, rect, color);

    f.render_widget(HelpWidget {}, rect);
}
