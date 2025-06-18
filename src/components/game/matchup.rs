use crate::components::constants::TEAM_IDS;
use crate::components::standings::Team;
use mlb_api::live::{FullPlayer, LiveResponse};
use mlb_api::plays::{Count, Play};
use std::collections::HashMap;
use std::fmt;
use tui::prelude::Stylize;
use tui::text::Line;

const DEFAULT_NAME: &str = "-";

pub struct Summary {
    pub home_team: Team,
    pub away_team: Team,
    pub pitcher: Option<Player>,
    pub batter: Option<Player>,
    pub on_deck: Option<String>,
    pub in_hole: Option<String>,
}

impl Default for Summary {
    fn default() -> Self {
        let home_team = Team {
            abbreviation: "H",
            ..Team::default()
        };
        let away_team = Team {
            abbreviation: "A",
            ..Team::default()
        };

        Self {
            home_team,
            away_team,
            pitcher: None,
            batter: None,
            on_deck: None,
            in_hole: None,
        }
    }
}

impl From<&LiveResponse> for Summary {
    fn from(live_game: &LiveResponse) -> Self {
        Self {
            home_team: TEAM_IDS
                .get(live_game.game_data.teams.home.name.as_str())
                .cloned()
                .unwrap_or_default(),
            away_team: TEAM_IDS
                .get(live_game.game_data.teams.away.name.as_str())
                .cloned()
                .unwrap_or_default(),
            pitcher: None, // TODO
            batter: None,  // TODO
            on_deck: live_game
                .live_data
                .linescore
                .offense
                .on_deck
                .as_ref()
                .map(|od| od.full_name.clone()),
            in_hole: live_game
                .live_data
                .linescore
                .offense
                .in_hole
                .as_ref()
                .map(|ih| ih.full_name.clone()),
        }
    }
}

#[derive(Clone, Debug)] // TODO remove clone
pub struct Player {
    pub id: u64,
    pub team_id: u16,
    pub first_name: String,
    /// First name that should be used for display.
    pub use_name: String,
    pub last_name: String,
    /// Last name that should be used for display.
    pub use_last_name: String,
    pub boxscore_name: String,
    pub summary: Option<String>,
    pub note: Option<String>,
}

impl From<&FullPlayer> for Player {
    fn from(player: &FullPlayer) -> Self {
        Self {
            id: player.id,
            team_id: 0, // TODO only from the boxscore data
            first_name: player.first_name.clone(),
            use_name: player.use_name.clone(),
            last_name: player.last_name.clone(),
            use_last_name: player.use_last_name.clone(),
            boxscore_name: player.boxscore_name.clone(),
            summary: None,
            note: None, // TODO more stuff
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            team_id: 0,
            first_name: DEFAULT_NAME.to_owned(),
            use_name: DEFAULT_NAME.to_owned(),
            last_name: DEFAULT_NAME.to_owned(),
            use_last_name: DEFAULT_NAME.to_owned(),
            boxscore_name: DEFAULT_NAME.to_owned(),
            summary: None,
            note: None,
        }
    }
}

pub struct MatchupV2 {
    #[allow(dead_code)]
    pub at_bat_index: u8,
    pub home_score: u8,
    pub away_score: u8,
    pub inning: u8,
    pub is_top: bool,
    pub pitcher: Player,
    pub pitcher_side: String,
    pub batter: Player,
    pub batter_side: String,
    pub count: Count,
    pub runners: Runners,
}

pub struct Matchup {
    pub home_name: String,
    pub home_score: u8,
    pub away_name: String,
    pub away_score: u8,
    pub inning: u8,
    pub is_top: bool,
    pub is_current_play: bool,
    pub pitcher: Player,
    pub pitcher_side: String,
    pub batter: Player,
    pub batter_side: String,
    pub count: Count,
    pub runners: Runners,
    pub on_deck: Option<String>,
    pub in_hole: Option<String>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Runners {
    pub first: bool,
    pub second: bool,
    pub third: bool,
}

impl fmt::Display for Runners {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut runners = String::new();
        if self.first {
            runners.push_str("1st ");
        }
        if self.second {
            runners.push_str("2nd ");
        }
        if self.third {
            runners.push_str("3rd");
        }
        write!(f, "{}", runners)
    }
}

impl Runners {
    pub fn from_matchup(matchup: &mlb_api::plays::Matchup) -> Self {
        Runners {
            first: matchup.post_on_first.is_some(),
            second: matchup.post_on_second.is_some(),
            third: matchup.post_on_third.is_some(),
        }
    }
}

impl Default for Matchup {
    fn default() -> Self {
        Matchup {
            home_name: DEFAULT_NAME.to_owned(),
            home_score: 0,
            away_name: DEFAULT_NAME.to_owned(),
            away_score: 0,
            inning: 1,
            is_top: true,
            is_current_play: false,
            pitcher_side: DEFAULT_NAME.to_owned(),
            batter_side: DEFAULT_NAME.to_owned(),
            count: Count::default(),
            runners: Runners::default(),
            on_deck: None,
            in_hole: None,
            batter: Player::default(),
            pitcher: Player::default(),
        }
    }
}

impl Default for MatchupV2 {
    fn default() -> Self {
        Self {
            at_bat_index: 0,
            home_score: 0,
            away_score: 0,
            inning: 1,
            is_top: true,
            pitcher: Player::default(),
            pitcher_side: DEFAULT_NAME.to_owned(),
            batter: Player::default(),
            batter_side: DEFAULT_NAME.to_owned(),
            count: Count::default(),
            runners: Runners::default(),
        }
    }
}

impl MatchupV2 {
    pub fn from(play: &Play, players: &HashMap<u64, Player>) -> Self {
        Self {
            at_bat_index: play.about.at_bat_index,
            home_score: play.result.home_score.unwrap_or(0),
            away_score: play.result.away_score.unwrap_or(0),
            inning: play.about.inning,
            is_top: play.about.is_top_inning,
            pitcher: players
                .get(&play.matchup.pitcher.id)
                .cloned()
                .unwrap_or_default(),
            batter: players
                .get(&play.matchup.batter.id)
                .cloned()
                .unwrap_or_default(),
            pitcher_side: format!("{}HP", play.matchup.pitch_hand.code.clone()),
            batter_side: play.matchup.bat_side.code.clone(),
            count: play.count.clone(),
            runners: Runners::from_matchup(&play.matchup),
        }
    }
}
// impl From<&Play> for MatchupV2 {
//     fn from(play: &Play) -> Self {
//         Self {
//             at_bat_index: play.about.at_bat_index,
//             home_score: play.result.home_score.unwrap_or(0),
//             away_score: play.result.away_score.unwrap_or(0),
//             inning: play.about.inning,
//             is_top: play.about.is_top_inning,
//             pitcher: Player {
//                 id: play.matchup.pitcher.id,
//                 team_id: 0,
//                 first_name: "".to_string(),
//                 use_name: "".to_string(),
//                 last_name: "".to_string(),
//                 use_last_name: "".to_string(),
//                 summary: None,
//                 note: None,
//             },
//             batter: Player::default(),
//             pitcher_side: format!("{}HP", play.matchup.pitch_hand.code.clone()),
//             batter_side: play.matchup.bat_side.code.clone(),
//             count: play.count.clone(),
//             runners: Runners::from_matchup(&play.matchup),
//         }
//     }
// }

impl Matchup {
    const ON_BASE_CHAR: char = '■';
    const EMPTY_BASE_CHAR: char = '□';

    pub fn from_v2(
        matchup: &MatchupV2,
        home_team: Team,
        away_team: Team,
        is_current: bool,
    ) -> Self {
        // hide on deck and in hole if not the current at bat since that info is only available for
        // the current at bat
        let (on_deck, in_hole) = if is_current {
            // (summary.on_deck.clone(), summary.in_hole.clone())
            (None, None)
        } else {
            (None, None)
        };
        Self {
            home_name: home_team.team_name.to_string(),
            home_score: matchup.home_score,
            away_name: away_team.team_name.to_string(),
            away_score: matchup.away_score,
            inning: matchup.inning,
            is_top: matchup.is_top,
            is_current_play: is_current,
            pitcher: matchup.pitcher.clone(),
            pitcher_side: matchup.pitcher_side.clone(),
            batter: matchup.batter.clone(),
            batter_side: matchup.batter_side.clone(),
            count: matchup.count.clone(),
            runners: matchup.runners,
            on_deck,
            in_hole,
        }
    }

    pub fn to_table_home(&self) -> Vec<Line> {
        let mut lines = vec![Line::from(self.home_name.to_string()).bold()];
        if self.is_top {
            lines.push(Line::from(format!(
                "{} {} - {}",
                self.pitcher.use_name, self.pitcher.use_last_name, self.pitcher_side
            )));
            if self.is_current_play {
                lines.push(Line::from(self.pitcher.summary.clone().unwrap_or_default()));
            }
            if self.pitcher.note.is_some() {
                lines.push(Line::from(self.pitcher.note.clone().unwrap_or_default()));
            }
        } else {
            lines.push(Line::from(format!(
                "{} {} - {}",
                self.batter.use_name, self.batter.use_last_name, self.batter_side
            )));
            if self.is_current_play {
                lines.push(Line::from(self.batter.summary.clone().unwrap_or_default()));
            }
            if self.batter.note.is_some() {
                lines.push(Line::from(self.batter.note.clone().unwrap_or_default()));
            }
        }
        lines
    }

    pub fn to_table_away(&self) -> Vec<Line> {
        let mut lines = vec![Line::from(self.away_name.to_string()).bold()];
        if self.is_top {
            lines.push(Line::from(format!(
                "{} {} - {}",
                self.batter.use_name, self.batter.use_last_name, self.batter_side
            )));
            if self.is_current_play {
                lines.push(Line::from(self.batter.summary.clone().unwrap_or_default()));
            }
            if self.batter.note.is_some() {
                lines.push(Line::from(self.batter.note.clone().unwrap_or_default()));
            }
        } else {
            lines.push(Line::from(format!(
                "{} {} - {}",
                self.pitcher.use_name, self.pitcher.use_last_name, self.pitcher_side
            )));
            if self.is_current_play {
                lines.push(Line::from(self.pitcher.summary.clone().unwrap_or_default()));
            }
            if self.pitcher.note.is_some() {
                lines.push(Line::from(self.pitcher.note.clone().unwrap_or_default()));
            }
        }
        lines
    }

    pub fn to_at_bat(&self) -> Vec<Line> {
        let outs = match self.count.outs {
            0 => "◯ ◯ ◯",
            1 => "● ◯ ◯",
            2 => "● ● ◯",
            3 => "● ● ●",
            _ => "",
        };
        // let second_base = match (self.runners.first, self.runners.second, self.runners.third) {
        //     (_, true, _) => "  ⬥  ",
        //     _ => "  ⬦  ",
        // };
        // let runners = match (self.runners.first, self.runners.third) {
        //     (false, false) => "⬦   ⬦",
        //     (true, false) => "⬥   ⬦",
        //     (true, true) => "⬥   ⬥",
        //     (false, true) => "⬦   ⬥",
        // };
        let second_base = match self.runners.second {
            true => format!("  {}  ", Self::ON_BASE_CHAR),
            false => format!("  {}  ", Self::EMPTY_BASE_CHAR),
        };
        let first_third = match (self.runners.first, self.runners.third) {
            (false, false) => format!("{}   {}", Self::EMPTY_BASE_CHAR, Self::EMPTY_BASE_CHAR),
            (true, false) => format!("{}   {}", Self::EMPTY_BASE_CHAR, Self::ON_BASE_CHAR),
            (false, true) => format!("{}   {}", Self::ON_BASE_CHAR, Self::EMPTY_BASE_CHAR),
            (true, true) => format!("{}   {}", Self::ON_BASE_CHAR, Self::ON_BASE_CHAR),
        };

        let arrow = if self.is_top { "▲" } else { "▼" };
        let info = vec![
            // TODO make the score bold
            Line::from(format!(
                "{}    {} {}    {}",
                self.away_score, arrow, self.inning, self.home_score
            )),
            Line::from(second_base),
            Line::from(first_third),
            Line::from(outs),
            // Line::from(format!("{}-{}", self.count.balls, self.count.strikes)),
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

// fn align_line(line: impl Into<String>, alignment: Alignment) -> Vec<Line<'static>> {
//     vec![Line::from(line.into()).alignment(alignment)]
// }

#[test]
fn test_matchup_default_runners() {
    // verify that the default is to have no runners on base
    let r = Runners::default();
    assert!(!r.first);
    assert!(!r.second);
    assert!(!r.third);
}

#[test]
#[rustfmt::skip]
fn test_matchup_runners_display() {
    // test that the runners are displayed correctly
    let on_first = Runners{first: true, second: false, third: false};
    assert_eq!("1st ".to_string(), on_first.to_string());

    let on_second = Runners{first: false, second: true, third: false};
    assert_eq!("2nd ".to_string(), on_second.to_string());

    let on_third = Runners{first: false, second: false, third: true};
    assert_eq!("3rd".to_string(), on_third.to_string());

    let first_second = Runners{first: true, second: true, third: false};
    assert_eq!("1st 2nd ".to_string(), first_second.to_string());

    let first_third = Runners{first: true, second: false, third: true};
    assert_eq!("1st 3rd".to_string(), first_third.to_string());

    let second_third = Runners{first: false, second: true, third: true};
    assert_eq!("2nd 3rd".to_string(), second_third.to_string());

    let loaded = Runners{first: true, second: true, third: true};
    assert_eq!("1st 2nd 3rd".to_string(), loaded.to_string());
}
