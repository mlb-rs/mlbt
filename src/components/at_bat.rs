use crate::components::pitches::Pitches;
use crate::components::strikezone::StrikeZone;
use mlb_api::live::LiveResponse;
use mlb_api::plays::Play;

#[derive(Default)]
pub struct AtBat {
    pub pitches: Pitches,
    pub strike_zone: StrikeZone,
}

impl AtBat {
    pub fn from_live_data(live_game: &LiveResponse) -> Self {
        AtBat {
            pitches: Pitches::from_live_data(live_game),
            strike_zone: StrikeZone::from_live_data(live_game),
        }
    }

    pub fn from_play(play: &Play) -> Self {
        AtBat {
            pitches: Pitches::from_play(play),
            strike_zone: StrikeZone::from_play(play),
        }
    }
}
