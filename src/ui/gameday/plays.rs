use crate::components::game::live_game::GameStateV2;
use crate::components::game::plays::PlayResult;
use crate::ui::layout::LayoutAreas;
use std::vec;
use tui::prelude::*;
use tui::widgets::{Paragraph, Wrap};

// These colors match the red, green, and blue used in the pitch data from the API.
pub const RED: Color = Color::Rgb(170, 21, 11);
pub const GREEN: Color = Color::Rgb(39, 161, 39);
// This matches the blue used in the pitch data from the API. It's used for contact (hit, out, run
// scoring).
pub const BLUE: Color = Color::Rgb(26, 86, 190);
pub const SCORING_SYMBOL: char = '!';
pub const SELECTION_SYMBOL: char = '>';

pub struct InningPlaysWidget<'a> {
    pub game: &'a GameStateV2,
    pub selected_at_bat: Option<u8>,
}

impl Widget for InningPlaysWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = LayoutAreas::for_info(area);

        let inning_plays = format_plays(self.game, self.selected_at_bat);
        let paragraph = Paragraph::new(inning_plays).wrap(Wrap { trim: false });

        Widget::render(paragraph, chunks[1], buf);
    }
}

/// Format the plays for the current half inning as TUI Lines.
fn format_plays(game: &GameStateV2, selected_at_bat: Option<u8>) -> Vec<Line> {
    let (at_bat, _is_current) = game.get_at_bat_by_index_or_current(selected_at_bat);
    let inning = at_bat.inning;
    let is_top_inning = at_bat.is_top_inning;

    if inning == 0 {
        return vec![];
    }

    // ensure the events are sorted by index, smallest first since it gets reversed later
    game.at_bats
        .values()
        .filter_map(|play| {
            if play.inning == inning && play.is_top_inning == is_top_inning {
                build_line(&play.play_result, selected_at_bat)
            } else {
                None
            }
        })
        .rev()
        .collect()
}

fn build_line(play: &PlayResult, selected_at_bat: Option<u8>) -> Option<Line> {
    let description = if play.description.is_empty() {
        "in progress..."
    } else {
        play.description.as_str()
    };
    let info = vec![
        format_runs(play, selected_at_bat),
        format_score(play),
        Span::raw(" "),
        Span::raw(description),
        format_outs(play),
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
        match selected {
            true => Span::raw(SELECTION_SYMBOL.to_string()).fg(BLUE).bold(),
            false => Span::raw("-"),
        }
    }
}

/// If runs were scored display the new score.
fn format_score(play: &PlayResult) -> Span {
    if play.is_scoring_play {
        Span::raw(format!(" {}-{}", play.away_score, play.home_score))
    } else {
        Span::raw("")
    }
}

/// If an out was made display it.
fn format_outs(play: &PlayResult) -> Span {
    if play.is_out {
        let out = if play.count.outs == 1 { "out" } else { "outs" };
        Span::raw(format!(" {} {}", &play.count.outs, out))
    } else {
        Span::raw("")
    }
}
