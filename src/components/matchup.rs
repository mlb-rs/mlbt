use mlb_api::live::LiveResponse;
use mlb_api::plays::Count;

use std::fmt;

const DEFAULT_NAME: &str = "-";

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

#[derive(Debug, Default)]
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
            home_name: DEFAULT_NAME.to_string(),
            home_score: 0,
            away_name: DEFAULT_NAME.to_string(),
            away_score: 0,
            inning: DEFAULT_NAME.to_string(),
            pitcher_name: DEFAULT_NAME.to_string(),
            pitcher_side: DEFAULT_NAME.to_string(),
            batter_name: DEFAULT_NAME.to_string(),
            batter_side: DEFAULT_NAME.to_string(),
            count: Count {
                strikes: 0,
                balls: 0,
                outs: 0,
            },
            runners: Runners::default(),
            on_deck: DEFAULT_NAME.to_string(),
            in_hole: DEFAULT_NAME.to_string(),
        }
    }
}

impl Matchup {
    pub fn from_live_data(live_game: &LiveResponse) -> Matchup {
        // Extract the current play from the API data. Not sure yet why the current play isn't
        // present. Maybe when the game isn't being played yet?
        let current = match live_game.live_data.plays.current_play.as_ref() {
            Some(c) => c,
            None => return Matchup::default(),
        };

        // get the on deck and in the hole batters
        let od = match live_game.live_data.linescore.offense.on_deck.as_ref() {
            Some(od) => od.full_name.clone(),
            None => DEFAULT_NAME.to_string(),
        };
        let ih = match live_game.live_data.linescore.offense.in_hole.as_ref() {
            Some(ih) => ih.full_name.clone(),
            None => DEFAULT_NAME.to_string(),
        };

        Matchup {
            home_name: live_game.game_data.teams.home.team_name.clone(),
            home_score: current.result.home_score.unwrap_or(0),
            away_name: live_game.game_data.teams.away.team_name.clone(),
            away_score: current.result.away_score.unwrap_or(0),
            inning: format!("{} {}", current.about.half_inning, current.about.inning),
            pitcher_name: current.matchup.pitcher.full_name.clone(),
            pitcher_side: format!("{}HP", current.matchup.pitch_hand.code.clone()),
            batter_name: current.matchup.batter.full_name.clone(),
            batter_side: current.matchup.bat_side.code.clone(),
            count: current.count.clone(),
            runners: Runners::from_matchup(&current.matchup),
            on_deck: od,
            in_hole: ih,
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
