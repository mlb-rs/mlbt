use mlb_api::live::LiveResponse;
use mlb_api::plays::{Count, Play};

use crate::components::constants::TEAM_IDS;
use crate::components::standings::Team;
use std::fmt;

const DEFAULT_NAME: &str = "-";

pub struct Summary {
    pub home_team: Team,
    pub away_team: Team,
    pub on_deck: String,
    pub in_hole: String,
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
            on_deck: DEFAULT_NAME.to_owned(),
            in_hole: DEFAULT_NAME.to_owned(),
        }
    }
}

impl From<&LiveResponse> for Summary {
    fn from(live_game: &LiveResponse) -> Self {
        // get the on deck and in the hole batters
        let on_deck = match live_game.live_data.linescore.offense.on_deck.as_ref() {
            Some(od) => od.full_name.clone(),
            None => DEFAULT_NAME.to_owned(),
        };
        let in_hole = match live_game.live_data.linescore.offense.in_hole.as_ref() {
            Some(ih) => ih.full_name.clone(),
            None => DEFAULT_NAME.to_owned(),
        };

        Self {
            home_team: TEAM_IDS
                .get(live_game.game_data.teams.home.name.as_str())
                .cloned()
                .unwrap_or_default(),
            away_team: TEAM_IDS
                .get(live_game.game_data.teams.away.name.as_str())
                .cloned()
                .unwrap_or_default(),
            on_deck,
            in_hole,
        }
    }
}

pub struct MatchupV2 {
    #[allow(dead_code)]
    pub at_bat_index: u8,
    pub home_score: u8,
    pub away_score: u8,
    pub inning: String,
    pub pitcher_name: String,
    pub pitcher_side: String,
    pub batter_name: String,
    pub batter_side: String,
    pub count: Count,
    pub runners: Runners,
}

pub struct Matchup {
    pub home_name: String,
    pub home_score: u8,
    pub away_name: String,
    pub away_score: u8,
    pub inning: String,
    pub pitcher_name: String,
    pub pitcher_side: String,
    pub batter_name: String,
    pub batter_side: String,
    pub count: Count,
    pub runners: Runners,
    pub on_deck: String,
    pub in_hole: String,
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
            inning: DEFAULT_NAME.to_owned(),
            pitcher_name: DEFAULT_NAME.to_owned(),
            pitcher_side: DEFAULT_NAME.to_owned(),
            batter_name: DEFAULT_NAME.to_owned(),
            batter_side: DEFAULT_NAME.to_owned(),
            count: Count::default(),
            runners: Runners::default(),
            on_deck: DEFAULT_NAME.to_owned(),
            in_hole: DEFAULT_NAME.to_owned(),
        }
    }
}

impl Default for MatchupV2 {
    fn default() -> Self {
        Self {
            at_bat_index: 0,
            home_score: 0,
            away_score: 0,
            inning: DEFAULT_NAME.to_owned(),
            pitcher_name: DEFAULT_NAME.to_owned(),
            pitcher_side: DEFAULT_NAME.to_owned(),
            batter_name: DEFAULT_NAME.to_owned(),
            batter_side: DEFAULT_NAME.to_owned(),
            count: Count::default(),
            runners: Runners::default(),
        }
    }
}

impl From<&Play> for MatchupV2 {
    fn from(play: &Play) -> Self {
        Self {
            at_bat_index: play.about.at_bat_index,
            home_score: play.result.home_score.unwrap_or(0),
            away_score: play.result.away_score.unwrap_or(0),
            inning: format!("{} {}", play.about.half_inning, play.about.inning),
            pitcher_name: play.matchup.pitcher.full_name.clone(),
            pitcher_side: format!("{}HP", play.matchup.pitch_hand.code.clone()),
            batter_name: play.matchup.batter.full_name.clone(),
            batter_side: play.matchup.bat_side.code.clone(),
            count: play.count.clone(),
            runners: Runners::from_matchup(&play.matchup),
        }
    }
}

impl Matchup {
    pub fn from_v2(matchup: &MatchupV2, summary: &Summary, is_current: bool) -> Self {
        // hide on deck and in hole if not the current at bat since that info is only available for
        // the current at bat
        let (on_deck, in_hole) = if is_current {
            (summary.on_deck.clone(), summary.in_hole.clone())
        } else {
            ("".to_string(), "".to_string())
        };
        Self {
            home_name: summary.home_team.team_name.to_string(),
            home_score: matchup.home_score,
            away_name: summary.away_team.team_name.to_string(),
            away_score: matchup.away_score,
            inning: matchup.inning.clone(),
            pitcher_name: matchup.pitcher_name.clone(),
            pitcher_side: matchup.pitcher_side.clone(),
            batter_name: matchup.batter_name.clone(),
            batter_side: matchup.batter_side.clone(),
            count: matchup.count.clone(),
            runners: matchup.runners,
            on_deck,
            in_hole,
        }
    }

    pub fn to_table(&self) -> Vec<Vec<String>> {
        vec![
            vec![self.away_name.clone(), self.away_score.to_string()],
            vec![self.home_name.clone(), self.home_score.to_string()],
            vec!["inning".to_string(), self.inning.clone()],
            vec!["outs".to_string(), self.count.outs.to_string()],
            vec!["balls".to_string(), self.count.balls.to_string()],
            vec!["strikes".to_string(), self.count.strikes.to_string()],
            vec![
                "pitcher".to_string(),
                format!("{} - {}", self.pitcher_name, self.pitcher_side),
            ],
            vec![
                "batter".to_string(),
                format!("{} - {}", self.batter_name, self.batter_side),
            ],
            vec!["runners".to_string(), self.runners.to_string()],
            vec!["on deck".to_string(), self.on_deck.clone()],
            vec!["in hole".to_string(), self.in_hole.clone()],
        ]
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
