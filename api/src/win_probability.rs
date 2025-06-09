use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WinProbabilityResponse {
    pub at_bats: Vec<WinProbabilityPerAtBat>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct About {
    pub at_bat_index: u16,
    pub is_top_inning: bool,
    pub inning: u8,
    pub captivating_index: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WinProbabilityPerAtBat {
    pub about: About,
    pub home_team_win_probability: f32,
    pub away_team_win_probability: f32,
    pub home_team_win_probability_added: f32,
    pub leverage_index: Option<f32>,
    pub at_bat_index: u16,
}
