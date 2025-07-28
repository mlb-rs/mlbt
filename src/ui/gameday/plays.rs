use crate::components::game::live_game::GameState;
use crate::components::game::plays::PlayResult;
use std::vec;
use tui::prelude::*;
use tui::widgets::{Paragraph, Wrap};

// These colors match the green and blue used in the pitch data from the API.
// The green is used for pitches called as balls.
// The red is used for pitches called as strikes.
// The blue is used for contact (hit, out, run scoring).
pub const GREEN: Color = Color::Rgb(39, 161, 39);
pub const BLUE: Color = Color::Rgb(26, 86, 190);
pub const RED: Color = Color::Rgb(170, 21, 11);
pub const SCORING_SYMBOL: char = '!';
pub const SELECTION_SYMBOL: char = '>';

pub struct InningPlaysWidget<'a> {
    pub game: &'a GameState,
    pub selected_at_bat: Option<u8>,
}

impl Widget for InningPlaysWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO this doesn't scroll properly. needs to be a list for that
        let inning_plays = format_plays(self.game, self.selected_at_bat);
        let paragraph = Paragraph::new(inning_plays).wrap(Wrap { trim: false });

        Widget::render(paragraph, area, buf);
    }
}

/// Format the plays for the current inning as TUI Lines.
fn format_plays(game: &GameState, selected_at_bat: Option<u8>) -> Vec<Line> {
    let (at_bat, _is_current) = game.get_at_bat_by_index_or_current(selected_at_bat);
    let inning = at_bat.inning;

    if inning == 0 {
        return vec![];
    }

    let mut lines = Vec::new();

    // Track last inning and top/bottom half
    let mut last_inning: Option<(bool, u8)> = None;

    for play in game.at_bats.values().rev() {
        let current_inning = (play.is_top_inning, play.inning);
        if play.inning != inning {
            continue;
        }

        if last_inning != Some(current_inning) {
            // Insert blank line before each new inning section except possibly at start
            if !lines.is_empty() {
                lines.push(Line::from(vec![]));
            }
            let half_str = if play.is_top_inning { "top" } else { "bottom" };
            let header_text = format!("## {} {}", half_str, play.inning);
            lines.push(Line::from(header_text).bold());
            last_inning = Some(current_inning);
        }

        if let Some(line) = build_line(
            &play.play_result,
            selected_at_bat,
            game.home_team.abbreviation,
            game.away_team.abbreviation,
        ) {
            lines.push(line);
        }
    }

    lines
}

fn build_line<'a>(
    play: &'a PlayResult,
    selected_at_bat: Option<u8>,
    home_team_abbreviation: &'static str,
    away_team_abbreviation: &'static str,
) -> Option<Line<'a>> {
    let description = if play.description.is_empty() {
        "in progress..."
    } else {
        play.description.as_str()
    };
    let info = vec![
        format_runs(play, selected_at_bat),
        Span::raw(" "),
        Span::raw(description),
        format_outs(play),
        format_score(play, home_team_abbreviation, away_team_abbreviation),
    ];
    Some(Line::from(info))
}

/// If runs were scored display as blue exclamation mark(s). Otherwise use `-` to indicate a new
/// line. If the line is selected, display `>` instead of `-`.
fn format_runs(play: &PlayResult, selected_at_bat: Option<u8>) -> Span {
    let selected = selected_at_bat
        .map(|ab_idx| play.at_bat_index == ab_idx)
        .unwrap_or(false);
    if play.is_scoring_play {
        // there could be no rbis on certain plays like a wild pitch but `!` should still be shown
        let runs = if play.rbi == 0 { 1 } else { play.rbi as usize };
        let rbis = SCORING_SYMBOL.to_string().repeat(runs);
        let text = match selected {
            true => format! {"{SELECTION_SYMBOL} {rbis}"},
            false => rbis,
        };
        Span::styled(text.to_string(), Style::default().fg(BLUE))
    } else {
        let mut color = Color::White;
        if play.is_out {
            color = RED;
        }
        let code = &play
            .events
            .last()
            .and_then(|last_event| last_event.code.as_deref());
        // hit
        if let Some("D") = code {
            color = BLUE;
        }
        // hbp
        if let Some("H") = code {
            color = GREEN;
        }
        if play.count.balls == 4 {
            color = GREEN;
        } else if play.count.strikes == 3 {
            color = RED;
        }
        match selected {
            true => Span::raw(SELECTION_SYMBOL.to_string()).fg(color).bold(),
            false => Span::raw("-").fg(color),
        }
    }
}

/// If runs were scored display the new score.
fn format_score<'a>(
    play: &'a PlayResult,
    home_team_abbreviation: &'static str,
    away_team_abbreviation: &'static str,
) -> Span<'a> {
    if play.is_scoring_play {
        build_scoring_span(
            play.home_score,
            home_team_abbreviation,
            play.away_score,
            away_team_abbreviation,
        )
    } else {
        Span::raw("")
    }
}

pub fn build_scoring_span(
    home_score: u8,
    home_team_abbreviation: &'static str,
    away_score: u8,
    away_team_abbreviation: &'static str,
) -> Span<'static> {
    Span::raw(format!(
        " [{away_team_abbreviation} {away_score}, {home_team_abbreviation} {home_score}]"
    ))
    .bold()
}

/// If an out was made display it.
fn format_outs(play: &PlayResult) -> Span {
    if play.is_out {
        let out = if play.count.outs == 1 { "out" } else { "outs" };
        Span::raw(format!(" {} {}", &play.count.outs, out)).bold()
    } else {
        Span::raw("")
    }
}
