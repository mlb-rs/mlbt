use crate::components::at_bat::AtBat;
use crate::components::boxscore::TeamBatterBoxscore;
use crate::components::linescore::LineScore;
use crate::components::matchup::Matchup;
use crate::components::plays::InningPlays;
use mlb_api::live::LiveResponse;

#[derive(Default)]
pub struct GameState {
    pub current_game_id: u64,
    pub linescore: LineScore,
    pub at_bat: AtBat,
    pub boxscore: TeamBatterBoxscore,
    pub matchup: Matchup,
    pub plays: InningPlays,
}

impl GameState {
    pub fn update(&mut self, live_data: &LiveResponse) {
        self.current_game_id = live_data.game_pk;
        self.linescore = LineScore::from_live_data(live_data);
        self.at_bat = AtBat::from_live_data(live_data);
        self.boxscore = TeamBatterBoxscore::from_live_data(live_data);
        self.matchup = Matchup::from_live_data(live_data);
        self.plays = InningPlays::from_live_data(live_data);
    }

    pub fn clear(&mut self) {
        self.current_game_id = 0;
        self.linescore = LineScore::default();
        self.at_bat = AtBat::default();
        self.boxscore = TeamBatterBoxscore::default();
        self.matchup = Matchup::default();
        self.plays = InningPlays::default();
    }

    pub fn get_current_game_id(&self) -> u64 {
        self.current_game_id
    }

    pub fn set_game_id(&mut self, game_id: u64) {
        self.current_game_id = game_id;
    }
}
