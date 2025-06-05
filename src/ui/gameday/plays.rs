use crate::components::game::live_game::GameStateV2;
use crate::components::game::plays::{InningPlays, PlayResult};
use crate::ui::layout::LayoutAreas;
use std::vec;
use tui::prelude::*;
use tui::widgets::{Paragraph, Wrap};

// This matches the blue used in the pitch data from the API. It's used for contact (hit, out, run
// scoring).
pub const BLUE: Color = Color::Rgb(26, 86, 190);
pub const SCORING_SYMBOL: char = '!';
pub const SELECTION_SYMBOL: char = '>';

impl InningPlays {
    pub fn as_lines(&self, selected_at_bat: Option<u8>) -> Vec<Line> {
        self.play_results
            .iter()
            .filter(|play| !play.description.is_empty())
            .map(|play| {
                let info = vec![
                    InningPlays::format_runs(play, selected_at_bat),
                    InningPlays::format_score(play),
                    Span::raw(" "),
                    Span::raw(&play.description),
                    InningPlays::format_outs(play),
                ];
                Line::from(info)
            })
            .rev()
            .collect()
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
}

pub struct InningPlaysWidget<'a> {
    pub game: &'a GameStateV2,
    pub selected_at_bat: Option<u8>,
}

impl Widget for InningPlaysWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = LayoutAreas::for_info(area);

        let inning_plays = InningPlays::from_gameday_v2(self.game, self.selected_at_bat);

        let text = Text::from(inning_plays.as_lines(self.selected_at_bat));
        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });

        Widget::render(paragraph, chunks[1], buf);
    }
}
