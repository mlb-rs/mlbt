use mlb_api::live::{Count, LiveResponse};
use std::fmt;

const DEFAULT_NAME: &str = "unknown";

pub struct Matchup {
    pub pitcher_name: String,
    pub batter_name: String,
    pub count: Count,
}

impl Default for Matchup {
    fn default() -> Self {
        Matchup {
            pitcher_name: DEFAULT_NAME.to_string(),
            batter_name: DEFAULT_NAME.to_string(),
            count: Count {
                strikes: 0,
                balls: 0,
                outs: 0,
            },
        }
    }
}

impl fmt::Display for Matchup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "pitcher: {}\nbatter: {}\n{}-{} {} out",
            self.pitcher_name,
            self.batter_name,
            self.count.balls,
            self.count.strikes,
            self.count.outs,
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

        // TODO probably don't need to clone all of these
        Matchup {
            pitcher_name: current.matchup.pitcher.full_name.clone(),
            batter_name: current.matchup.batter.full_name.clone(),
            count: current.count.clone(),
        }
    }
}

#[test]
fn test_string_display() {
    let matchup = Matchup {
        pitcher_name: "Nolan Ryan".to_string(),
        batter_name: "Sammy Sosa".to_string(),
        count: Count {
            balls: 3,
            strikes: 2,
            outs: 2,
        },
    };
    let w = format!(
        "pitcher: {}\nbatter: {}\n{}-{} {} out",
        matchup.pitcher_name,
        matchup.batter_name,
        matchup.count.balls,
        matchup.count.strikes,
        matchup.count.outs,
    );
    assert_eq!(w, matchup.to_string());
}
