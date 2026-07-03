use crate::components::game::live_game::GameState;
use crate::components::game::plays::PlayResult;
use crate::ui::scroll::render_scrollbar;
use crate::ui::styling::TEXT_COLOR;
use std::vec;
use tui::prelude::*;
use tui::widgets::{Paragraph, ScrollbarState, Wrap};

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
    pub scoring_only: bool,
    pub scroll_offset: &'a mut u16,
    pub scroll_state: &'a mut ScrollbarState,
}

impl Widget for InningPlaysWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (lines, selected_idx) =
            format_plays(self.game, self.selected_at_bat, self.scoring_only);

        let content_height = measure(&lines, area.width);
        let paragraph = Paragraph::new(lines.clone()).wrap(Wrap { trim: false });

        // Everything fits, so render without scrolling.
        if content_height <= area.height {
            *self.scroll_offset = 0;
            *self.scroll_state = ScrollbarState::default();
            Widget::render(paragraph, area, buf);
            return;
        }

        // Follow the selection so it stays on screen. Lines wrap, so measure its start row and height.
        let selected = selected_idx.map(|idx| {
            let start = measure(&lines[..idx], area.width);
            let height = measure(&lines[idx..=idx], area.width);
            (start, height)
        });
        let offset = follow_selection(*self.scroll_offset, selected, area.height, content_height);

        *self.scroll_offset = offset;
        *self.scroll_state = ScrollbarState::default()
            .content_length(content_height as usize)
            .position(offset as usize);

        Widget::render(paragraph.scroll((offset, 0)), area, buf);
        render_scrollbar(area, self.scroll_state, buf);
    }
}

/// Nudge the offset to keep the selection in view, moving only when it would fall off screen.
/// `content` and `viewport` are wrapped row counts. With no selection, stay at the top.
fn follow_selection(
    current: u16,
    selected: Option<(u16, u16)>,
    viewport: u16,
    content: u16,
) -> u16 {
    /// Rows of context kept around the selection so its inning header (the row above) stays visible.
    const SCROLL_MARGIN: u16 = 1;

    let max = content.saturating_sub(viewport);
    let Some((start, height)) = selected else {
        return 0;
    };
    let mut offset = current.min(max);
    if start < offset + SCROLL_MARGIN {
        offset = start.saturating_sub(SCROLL_MARGIN);
    } else if start + height + SCROLL_MARGIN > offset + viewport {
        offset = (start + height + SCROLL_MARGIN).saturating_sub(viewport);
    }
    offset.min(max)
}

/// Measure the wrapped height, in rows, that `lines` occupy at the given width.
fn measure(lines: &[Line<'_>], width: u16) -> u16 {
    Paragraph::new(lines.to_vec())
        .wrap(Wrap { trim: false })
        .line_count(width) as u16
}

/// Format the plays for the current inning as TUI Lines, returning the index of the line for the
/// selected at bat when one is shown.
fn format_plays(
    game: &GameState,
    selected_at_bat: Option<u8>,
    scoring_only: bool,
) -> (Vec<Line<'_>>, Option<usize>) {
    let (at_bat, _is_current) = game.get_at_bat_by_index_or_current(selected_at_bat);
    let inning = at_bat.inning;

    if inning == 0 {
        return (vec![], None);
    }

    let mut lines = Vec::new();
    let mut selected_idx = None;

    // Track last inning and top/bottom half
    let mut last_inning: Option<(bool, u8)> = None;

    for play in game.at_bats.values().rev() {
        let current_inning = (play.is_top_inning, play.inning);
        if scoring_only {
            if !play.play_result.is_scoring_play {
                continue;
            }
        } else if play.inning != inning {
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
            if selected_at_bat == Some(play.play_result.at_bat_index) {
                selected_idx = Some(lines.len());
            }
            lines.push(line);
        }
    }

    (lines, selected_idx)
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
fn format_runs(play: &PlayResult, selected_at_bat: Option<u8>) -> Span<'_> {
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
        let mut color = TEXT_COLOR;
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
fn format_outs(play: &PlayResult) -> Span<'_> {
    if play.is_out {
        let out = if play.count.outs == 1 { "out" } else { "outs" };
        Span::raw(format!(" {} {}", &play.count.outs, out)).bold()
    } else {
        Span::raw("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follow_selection_keeps_the_cursor_in_view() {
        // viewport of 10 rows over 30 rows of content, so the offset can range 0..=20
        let viewport = 10;
        let content = 30;

        // nothing selected parks the view at the top where the newest plays render
        assert_eq!(follow_selection(15, None, viewport, content), 0);

        // an already visible selection leaves the offset untouched
        assert_eq!(follow_selection(5, Some((8, 1)), viewport, content), 5);

        // a selection above the viewport scrolls up, keeping one row of header context above it
        assert_eq!(follow_selection(12, Some((4, 1)), viewport, content), 3);

        // a selection below the viewport scrolls down, leaving one row of context below it
        assert_eq!(follow_selection(0, Some((14, 2)), viewport, content), 7);

        // the offset never exceeds the max scroll, even chasing the last line
        assert_eq!(follow_selection(0, Some((29, 1)), viewport, content), 20);
    }
}
