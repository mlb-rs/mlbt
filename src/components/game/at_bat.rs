use crate::components::game::matchup::{MatchupV2, Player};
use crate::components::game::pitches::Pitches;
use crate::components::game::plays::PlayResult;
use crate::components::game::strikezone::StrikeZone;
use mlb_api::plays::Play;
use std::collections::HashMap;

#[derive(Default)]
pub struct AtBatV2 {
    pub index: u8,
    pub inning: u8,
    pub is_top_inning: bool,
    /// Strikezone, pitches, pitch information
    pub pitches: AtBatPitches,
    /// Matchup information
    pub matchup: MatchupV2,
    /// Play information
    pub play_result: PlayResult,
}

#[derive(Default)]
pub struct AtBatPitches {
    pub pitches: Pitches,
    pub strike_zone: StrikeZone,
}

impl AtBatV2 {
    pub fn from(play: &Play, players: &HashMap<u64, Player>) -> Self {
        let matchup = MatchupV2::from(play, players);
        let pitches = AtBatPitches::from(play);
        let play_result = PlayResult::from(play);
        Self {
            index: play.about.at_bat_index,
            inning: play.about.inning,
            is_top_inning: play.about.is_top_inning,
            pitches,
            matchup,
            play_result,
        }
    }
}
// impl From<&Play> for AtBatV2 {
//     fn from(play: &Play) -> Self {
//         Self {
//             index: play.about.at_bat_index,
//             inning: play.about.inning,
//             is_top_inning: play.about.is_top_inning,
//             pitches: play.into(),
//             matchup: play.into(),
//             play_result: play.into(),
//         }
//     }
// }

impl From<&Play> for AtBatPitches {
    fn from(play: &Play) -> Self {
        AtBatPitches {
            pitches: play.into(),
            strike_zone: play.into(),
        }
    }
}
