use mlb_api::live::{Count, LiveResponse};
use std::fmt;

const DEFAULT_NAME: &str = "unknown";

pub struct Matchup {
    pub inning: String,
    pub pitcher_name: String,
    pub pitcher_side: String,
    pub batter_name: String,
    pub batter_side: String,
    pub count: Count,
    pub runners: Runners,
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
    pub fn from_matchup(matchup: &mlb_api::live::Matchup) -> Self {
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
            inning: DEFAULT_NAME.to_string(),
            pitcher_name: DEFAULT_NAME.to_string(),
            pitcher_side: "R".to_string(),
            batter_name: DEFAULT_NAME.to_string(),
            batter_side: "R".to_string(),
            count: Count {
                strikes: 0,
                balls: 0,
                outs: 0,
            },
            runners: Runners::default(),
        }
    }
}

impl fmt::Display for Matchup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            " inning: {}\n pitching: {} - {}HP\n at bat:   {} - {}\n balls: {:>3}\n strikes: {}\n outs: {:>4}\n on base: {}",
            self.inning,
            self.pitcher_name,
            self.pitcher_side,
            self.batter_name,
            self.batter_side,
            self.count.balls,
            self.count.strikes,
            self.count.outs,
            self.runners,
        )
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

        Matchup {
            inning: format!("{} {}", current.about.half_inning, current.about.inning),
            pitcher_name: current.matchup.pitcher.full_name.clone(),
            pitcher_side: current.matchup.pitch_hand.code.clone(),
            batter_name: current.matchup.batter.full_name.clone(),
            batter_side: current.matchup.bat_side.code.clone(),
            count: current.count.clone(),
            runners: Runners::from_matchup(&current.matchup),
        }
    }
}

#[test]
fn test_matchup_string_display() {
    let matchup = Matchup {
        inning: "bottom 9".to_string(),
        pitcher_name: "Nolan Ryan".to_string(),
        pitcher_side: "R".to_string(),
        batter_name: "Sammy Sosa".to_string(),
        batter_side: "R".to_string(),
        count: Count {
            balls: 3,
            strikes: 2,
            outs: 2,
        },
        runners: Runners {
            first: true,
            second: true,
            third: true,
        },
    };
    let w = r" inning: bottom 9
 pitching: Nolan Ryan - RHP
 at bat:   Sammy Sosa - R
 balls:   3
 strikes: 2
 outs:    2
 on base: 1st 2nd 3rd"
        .to_string();
    assert_eq!(w, matchup.to_string());
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
