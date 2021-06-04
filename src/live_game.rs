use crate::at_bat::AtBat;
use crate::boxscore_stats::TeamBatterBoxscore;
use crate::linescore::LineScore;
use crate::matchup::Matchup;
use crate::plays::InningPlays;
use mlb_api::live::LiveResponse;

pub struct GameState {
    pub live_data: LiveResponse,
    pub linescore: LineScore,
    pub at_bat: AtBat,
    pub boxscore: TeamBatterBoxscore,
    pub matchup: Matchup,
    pub plays: InningPlays,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            live_data: LiveResponse::default(),
            linescore: LineScore::default(),
            at_bat: AtBat::default(),
            boxscore: TeamBatterBoxscore::default(),
            matchup: Matchup::default(),
            plays: InningPlays::default(),
        }
    }
}
