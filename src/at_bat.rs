use crate::pitches::Pitches;
use crate::strikezone::StrikeZone;
use mlb_api::live::LiveResponse;

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
}
