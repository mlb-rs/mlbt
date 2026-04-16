use crate::components::game::live_game::PlayerMap;
use crate::components::team_colors;
use mlbt_api::plays::{Count, Play};
use tui::prelude::{Span, Style, Stylize};
use tui::style::Color;
use tui::text::Line;

pub struct Matchup {
    #[allow(dead_code)]
    pub at_bat_index: u8,
    pub home_score: u8,
    pub away_score: u8,
    pub inning: u8,
    pub is_top: bool,
    pub pitcher_id: u64,
    pub batter_id: u64,
    pub count: Count,
    pub runners: Runners,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Runners {
    pub first: bool,
    pub second: bool,
    pub third: bool,
}

impl Runners {
    pub fn from_matchup(matchup: &mlbt_api::plays::Matchup) -> Self {
        Runners {
            first: matchup.post_on_first.is_some(),
            second: matchup.post_on_second.is_some(),
            third: matchup.post_on_third.is_some(),
        }
    }

    /// Generate two lines, one for second base and one for first and third. Second base is shown
    /// on a line above first and third.
    pub fn generate_lines(&self, symbols: &crate::symbols::Symbols) -> (Line<'_>, Line<'_>) {
        let on = symbols.base_occupied();
        let off = symbols.base_empty();
        let second_base = match self.second {
            true => format!("  {on}  "),
            false => format!("  {off}  "),
        };
        let first_third = match (self.first, self.third) {
            (false, false) => format!("{off}   {off}"),
            (true, false) => format!("{off}   {on}"),
            (false, true) => format!("{on}   {off}"),
            (true, true) => format!("{on}   {on}"),
        };
        (Line::from(second_base), Line::from(first_third))
    }
}

impl Default for Matchup {
    fn default() -> Self {
        Self {
            at_bat_index: 0,
            home_score: 0,
            away_score: 0,
            inning: 1,
            is_top: true,
            pitcher_id: 0,
            batter_id: 0,
            count: Count::default(),
            runners: Runners::default(),
        }
    }
}

impl From<&Play> for Matchup {
    fn from(play: &Play) -> Self {
        Self {
            at_bat_index: play.about.at_bat_index,
            home_score: play.result.home_score.unwrap_or(0),
            away_score: play.result.away_score.unwrap_or(0),
            inning: play.about.inning,
            is_top: play.about.is_top_inning,
            pitcher_id: play.matchup.pitcher.id,
            batter_id: play.matchup.batter.id,
            count: play.count.clone(),
            runners: Runners::from_matchup(&play.matchup),
        }
    }
}

impl Matchup {
    pub fn format_team_lines(
        &self,
        team_name: &str,
        abbreviation: &str,
        abs_challenges: Option<u8>,
        is_home: bool,
        current_play: bool,
        players: &PlayerMap,
        symbols: &crate::symbols::Symbols,
    ) -> Vec<Line<'_>> {
        // only show the remaining challenges if the current play is selected
        let challenges = if current_play {
            match abs_challenges {
                Some(0) => "◇ ◇",
                Some(1) => "◆ ◇",
                Some(2) => "◆ ◆",
                _ => "",
            }
        } else {
            ""
        };
        let header = match (is_home, challenges.is_empty()) {
            (true, false) => format!("{challenges}  {team_name}"),
            (false, false) => format!("{team_name}  {challenges}"),
            _ => team_name.to_string(),
        };
        let header_span = if symbols.team_colors() {
            team_colors::get(abbreviation, false)
                .map(|c| Span::raw(header.clone()).bold().fg(c))
                .unwrap_or_else(|| Span::raw(header.clone()).bold())
        } else {
            Span::raw(header.clone()).bold()
        };
        let mut lines = vec![Line::from(header_span)];

        let is_batting = if is_home { !self.is_top } else { self.is_top };
        if is_batting {
            lines.extend(self.get_batter_display_lines(current_play, players));
        } else {
            lines.extend(self.get_pitcher_display_lines(current_play, players));
        }
        lines
    }

    fn get_pitcher_display_lines(&self, current_play: bool, players: &PlayerMap) -> Vec<Line<'_>> {
        let pitcher = match players.get(&self.pitcher_id) {
            Some(p) => p,
            None => return vec![],
        };

        let mut lines = Vec::new();
        lines.push(Line::from(format!(
            "{} {} - {}",
            pitcher.first_name, pitcher.last_name, pitcher.pitch_hand
        )));
        if let Some(note) = &pitcher.stats.note {
            lines.push(Line::from(note.clone()));
        }
        if current_play {
            if pitcher.stats.pitches_thrown.is_some() && pitcher.stats.strikes.is_some() {
                lines.push(Line::from(format!(
                    "{} P - {} S",
                    pitcher.stats.pitches_thrown.unwrap_or_default(),
                    pitcher.stats.strikes.unwrap_or_default()
                )));
            }
            lines.push(Line::from(
                pitcher.stats.summary.clone().unwrap_or_default(),
            ));
        }
        lines
    }

    fn get_batter_display_lines(&self, current_play: bool, players: &PlayerMap) -> Vec<Line<'_>> {
        let batter = match players.get(&self.batter_id) {
            Some(p) => p,
            None => return vec![],
        };

        let mut lines = Vec::new();
        lines.push(Line::from(format!(
            "{} {} - {}",
            batter.first_name, batter.last_name, batter.batter_side
        )));
        if current_play {
            let summary = batter.stats.summary.clone().unwrap_or_default();
            let splits: Vec<String> = summary.split(" | ").map(|s| s.to_string()).collect();

            if let Some(ab) = splits.first() {
                lines.push(Line::from(ab.clone()));
            }
            if let Some(highlights) = splits.get(1) {
                lines.push(Line::from(highlights.clone()));
            } else {
                lines.push(Line::from("-"));
            }
        }
        lines
    }

    pub fn format_scoreboard_lines(&self, symbols: &crate::symbols::Symbols) -> Vec<Line<'_>> {
        let arrow = if self.is_top { "▲" } else { "▼" };
        let (second_base, first_third) = self.runners.generate_lines(symbols);

        let outs_line = if symbols.nerd_fonts() {
            let filled = Span::styled("●", Style::default().fg(Color::Red));
            let empty = Span::styled("◯", Style::default().fg(Color::DarkGray));
            let sp = Span::raw(" ");
            let count = self.count.outs as usize;
            let mut spans: Vec<Span<'_>> = Vec::with_capacity(5);
            for i in 0..3 {
                if i > 0 {
                    spans.push(sp.clone());
                }
                spans.push(if i < count {
                    filled.clone()
                } else {
                    empty.clone()
                });
            }
            Line::from(spans)
        } else {
            let s = match self.count.outs {
                0 => "◯ ◯ ◯",
                1 => "● ◯ ◯",
                2 => "● ● ◯",
                3 => "● ● ●",
                _ => "",
            };
            Line::from(s)
        };

        vec![
            Line::from(format!(
                "{}    {} {}    {}",
                self.away_score, arrow, self.inning, self.home_score
            )),
            second_base,
            first_third,
            outs_line,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn header(name: &str, challenges: Option<u8>, is_home: bool, current: bool) -> String {
        use crate::symbols::Symbols;
        use crate::theme::ThemeLevel;
        let m = Matchup::default();
        let symbols = Symbols::new(false, false, ThemeLevel::default());
        m.format_team_lines(name, "TST", challenges, is_home, current, &HashMap::new(), &symbols)[0]
            .to_string()
    }

    #[test]
    fn default_runners() {
        let r = Runners::default();
        assert!(!r.first && !r.second && !r.third);
    }

    #[test]
    fn home_challenges_before_name() {
        assert!(header("NYY", Some(2), true, true).starts_with("◆ ◆"));
    }

    #[test]
    fn away_challenges_after_name() {
        let h = header("BOS", Some(1), false, true);
        assert!(h.starts_with("BOS") && h.contains("◆ ◇"));
    }

    #[test]
    fn zero_challenges_shows_empty_diamonds() {
        assert!(header("CHC", Some(0), true, true).contains("◇ ◇"));
    }

    #[test]
    fn no_challenges_when_not_current_play() {
        assert_eq!(header("CHC", Some(2), true, false), "CHC");
    }

    #[test]
    fn no_challenges_when_none() {
        assert_eq!(header("CHC", None, true, true), "CHC");
    }
}
