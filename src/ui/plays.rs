use crate::components::plays::{InningPlays, PlayResult};
use crate::ui::layout::LayoutAreas;

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, StatefulWidget, Widget, Wrap},
};

// This matches the blue used in the pitch data from the API. It's used for contact (hit, out, run
// scoring).
pub const BLUE: Color = Color::Rgb(26, 86, 190);
pub const SCORING_SYMBOL: char = '!';

impl InningPlays {
    pub fn as_lines(&self) -> Vec<Line> {
        self.play_results
            .iter()
            .filter(|play| !play.description.is_empty())
            .map(|play| {
                let info = vec![
                    InningPlays::format_runs(play),
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

    // If runs were scored display as blue exclamation mark(s). Otherwise use `-` to indicate a new
    // line.
    fn format_runs(play: &PlayResult) -> Span {
        if play.is_scoring_play {
            // there could be no rbis on certain plays like a wild pitch but `!` should still be shown
            let runs = if play.rbi == 0 { 1 } else { play.rbi as usize };
            let rbis = SCORING_SYMBOL.to_string().repeat(runs);
            Span::styled(rbis.to_string(), Style::default().fg(BLUE))
        } else {
            Span::raw("-")
        }
    }

    // If runs were scored display the new score.
    fn format_score(play: &PlayResult) -> Span {
        if play.is_scoring_play {
            Span::raw(format!(" {}-{}", play.away_score, play.home_score))
        } else {
            Span::raw("")
        }
    }

    // If an out was made display it.
    fn format_outs(play: &PlayResult) -> Span {
        if play.is_out {
            let out = if play.count.outs == 1 { "out" } else { "outs" };
            Span::raw(format!(" {} {}", &play.count.outs, out))
        } else {
            Span::raw("")
        }
    }
}

pub struct InningPlaysWidget {}

impl StatefulWidget for InningPlaysWidget {
    type State = InningPlays;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunks = LayoutAreas::for_info(area);

        let text = Text::from(state.as_lines());
        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });

        Widget::render(paragraph, chunks[1], buf);
    }
}
