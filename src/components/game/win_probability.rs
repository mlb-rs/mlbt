use crate::components::game::live_game::AtBatIndex;
use indexmap::IndexMap;
use mlb_api::win_probability::{WinProbabilityPerAtBat, WinProbabilityResponse};

#[derive(Debug)]
pub struct WinProbability {
    pub at_bats: IndexMap<AtBatIndex, WinProbabilityAtBat>,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct WinProbabilityAtBat {
    pub at_bat_index: AtBatIndex,
    pub is_top_inning: bool,
    pub inning: u8,
    pub home_team_wp: f32,
    pub away_team_wp: f32,
    pub home_team_wp_added: f32,
    pub leverage_index: f32,
}

impl Default for WinProbability {
    fn default() -> Self {
        Self {
            at_bats: IndexMap::from([(0, WinProbabilityAtBat::default())]),
        }
    }
}

impl Default for WinProbabilityAtBat {
    fn default() -> Self {
        WinProbabilityAtBat {
            at_bat_index: 0,
            is_top_inning: true,
            inning: 1,
            home_team_wp: 50.0,
            away_team_wp: 50.0,
            home_team_wp_added: 0.0,
            leverage_index: 0.0,
        }
    }
}

impl From<&WinProbabilityPerAtBat> for WinProbabilityAtBat {
    fn from(at_bat: &WinProbabilityPerAtBat) -> Self {
        WinProbabilityAtBat {
            at_bat_index: at_bat.at_bat_index,
            is_top_inning: at_bat.about.is_top_inning,
            inning: at_bat.about.inning,
            home_team_wp: at_bat.home_team_win_probability,
            away_team_wp: at_bat.away_team_win_probability,
            home_team_wp_added: at_bat.home_team_win_probability_added,
            leverage_index: at_bat.leverage_index.unwrap_or(0.0),
        }
    }
}

impl From<&WinProbabilityResponse> for WinProbability {
    fn from(response: &WinProbabilityResponse) -> Self {
        let mut at_bats = IndexMap::new();
        for ab in &response.at_bats {
            at_bats.insert(ab.at_bat_index, WinProbabilityAtBat::from(ab));
        }
        if at_bats.is_empty() {
            at_bats.insert(0, WinProbabilityAtBat::default());
        }
        WinProbability { at_bats }
    }
}
