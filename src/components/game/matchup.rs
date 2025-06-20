use crate::components::game::live_game::PlayerStats;
use mlb_api::live::FullPlayer;
use mlb_api::plays::{Count, Play};
use std::collections::HashMap;
use tui::prelude::Stylize;
use tui::text::Line;

const DEFAULT_NAME: &str = "-";

#[derive(Debug)]
pub struct Player {
    #[allow(dead_code)]
    pub id: u64,
    pub first_name: String,
    pub last_name: String,
    pub boxscore_name: String,
    pub batter_side: String,
    pub pitch_hand: String,
    pub stats: PlayerStats,
}

impl From<&FullPlayer> for Player {
    fn from(player: &FullPlayer) -> Self {
        Self {
            id: player.id,
            first_name: player
                .use_name
                .clone()
                .unwrap_or_else(|| DEFAULT_NAME.to_owned()),
            last_name: player
                .use_last_name
                .clone()
                .unwrap_or_else(|| DEFAULT_NAME.to_owned()),
            boxscore_name: player
                .boxscore_name
                .clone()
                .unwrap_or_else(|| DEFAULT_NAME.to_owned()),
            batter_side: player
                .bat_side
                .as_ref()
                .map(|b| b.code.clone())
                .unwrap_or_default(),
            pitch_hand: player
                .pitch_hand
                .as_ref()
                .map(|b| format!("{}HP", b.code))
                .unwrap_or_default(),
            stats: PlayerStats::default(), // gets set later
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            first_name: DEFAULT_NAME.to_owned(),
            last_name: DEFAULT_NAME.to_owned(),
            boxscore_name: DEFAULT_NAME.to_owned(),
            batter_side: DEFAULT_NAME.to_owned(),
            pitch_hand: DEFAULT_NAME.to_owned(),
            stats: PlayerStats::default(),
        }
    }
}

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
    const ON_BASE_CHAR: char = '■'; // ⬥
    const EMPTY_BASE_CHAR: char = '□'; // ⬦ 

    pub fn from_matchup(matchup: &mlb_api::plays::Matchup) -> Self {
        Runners {
            first: matchup.post_on_first.is_some(),
            second: matchup.post_on_second.is_some(),
            third: matchup.post_on_third.is_some(),
        }
    }

    /// Generate two lines, one for second base and one for first and third. Second base is shown
    /// on a line above first and third.
    pub fn generate_lines(&self) -> (Line, Line) {
        let second_base = match self.second {
            true => format!("  {}  ", Self::ON_BASE_CHAR),
            false => format!("  {}  ", Self::EMPTY_BASE_CHAR),
        };
        let first_third = match (self.first, self.third) {
            (false, false) => format!("{}   {}", Self::EMPTY_BASE_CHAR, Self::EMPTY_BASE_CHAR),
            (true, false) => format!("{}   {}", Self::EMPTY_BASE_CHAR, Self::ON_BASE_CHAR),
            (false, true) => format!("{}   {}", Self::ON_BASE_CHAR, Self::EMPTY_BASE_CHAR),
            (true, true) => format!("{}   {}", Self::ON_BASE_CHAR, Self::ON_BASE_CHAR),
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
    pub fn format_home_lines(
        &self,
        home_name: &str,
        current_play: bool,
        players: &HashMap<u64, Player>,
    ) -> Vec<Line> {
        let mut lines = vec![Line::from(home_name.to_string()).bold()];
        if self.is_top {
            lines.extend(self.get_pitcher_display_lines(current_play, players));
        } else {
            lines.extend(self.get_batter_display_lines(current_play, players));
        }
        lines
    }

    pub fn format_away_lines(
        &self,
        away_name: &str,
        current_play: bool,
        players: &HashMap<u64, Player>,
    ) -> Vec<Line> {
        let mut lines = vec![Line::from(away_name.to_string()).bold()];
        if self.is_top {
            lines.extend(self.get_batter_display_lines(current_play, players));
        } else {
            lines.extend(self.get_pitcher_display_lines(current_play, players));
        }
        lines
    }

    fn get_pitcher_display_lines(
        &self,
        current_play: bool,
        players: &HashMap<u64, Player>,
    ) -> Vec<Line> {
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

    fn get_batter_display_lines(
        &self,
        current_play: bool,
        players: &HashMap<u64, Player>,
    ) -> Vec<Line> {
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
        // if let Some(note) = &batter.stats.note {
        //     lines.push(Line::from(note.clone()));
        // }
        lines
    }

    pub fn format_scoreboard_lines(&self) -> Vec<Line> {
        let outs = match self.count.outs {
            0 => "◯ ◯ ◯",
            1 => "● ◯ ◯",
            2 => "● ● ◯",
            3 => "● ● ●",
            _ => "",
        };
        let arrow = if self.is_top { "▲" } else { "▼" };
        let (second_base, first_third) = self.runners.generate_lines();

        let info = vec![
            Line::from(format!(
                "{}    {} {}    {}",
                self.away_score, arrow, self.inning, self.home_score
            )),
            second_base,
            first_third,
            Line::from(outs),
        ];
        // if !self.on_deck.is_empty() {
        //     info.push(Line::from(format!("on deck: {}", self.on_deck)));
        // }
        // if !self.in_hole.is_empty() {
        //     info.push(Line::from(format!("in hole: {}", self.in_hole)));
        // }

        info
    }
}

#[test]
fn test_matchup_default_runners() {
    // verify that the default is to have no runners on base
    let r = Runners::default();
    assert!(!r.first);
    assert!(!r.second);
    assert!(!r.third);
}
