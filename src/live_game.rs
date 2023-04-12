use crate::at_bat::AtBat;
use crate::boxscore::TeamBatterBoxscore;
use crate::linescore::LineScore;
use crate::matchup::Matchup;
use crate::plays::InningPlays;
use mlb_api::live::LiveResponse;

#[derive(Default)]
pub struct GameState {
    pub linescore: LineScore,
    pub at_bat: AtBat,
    pub boxscore: TeamBatterBoxscore,
    pub matchup: Matchup,
    pub plays: InningPlays,
}

impl GameState {
    pub fn update(&mut self, live_data: &LiveResponse) {
        self.linescore = LineScore::from_live_data(live_data);
        self.at_bat = AtBat::from_live_data(live_data);
        self.boxscore = TeamBatterBoxscore::from_live_data(live_data);
        self.matchup = Matchup::from_live_data(live_data);
        self.plays = InningPlays::from_live_data(live_data);
    }
}
