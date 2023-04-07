use crate::live::Person;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plays {
    pub all_plays: Option<Vec<Play>>,
    pub current_play: Option<Play>,
    pub scoring_plays: Option<Vec<u8>>,
    pub plays_by_inning: Option<Vec<PlaysByInning>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Play {
    pub result: Result,
    pub about: About,
    pub count: Count,
    pub matchup: Matchup,
    pub play_events: Vec<PlayEvent>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaysByInning {
    pub start_index: u8,
    pub end_index: u8,
    pub top: Vec<u8>,
    pub bottom: Vec<u8>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    // #[serde(rename = "type")]
    // pub result_type: Option<ResultType>,
    pub event: Option<String>,
    pub event_type: Option<String>,
    pub description: Option<String>,
    pub rbi: Option<u8>,
    pub away_score: Option<u8>,
    pub home_score: Option<u8>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct About {
    pub at_bat_index: u8,
    pub half_inning: String,
    pub is_top_inning: bool,
    pub inning: u8,
    pub is_complete: bool,
    pub is_scoring_play: Option<bool>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Matchup {
    pub batter: Person,
    pub bat_side: Side,
    pub pitcher: Person,
    pub pitch_hand: Side,
    pub batter_hot_cold_zones: Option<Vec<Zone>>,
    pub post_on_first: Option<Person>,
    pub post_on_second: Option<Person>,
    pub post_on_third: Option<Person>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Side {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zone {
    pub zone: String,
    pub color: String, // this is what I want: "rgba(255, 255, 255, 0.55)" -> need to convert it to a color
    pub value: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayEvent {
    pub details: Details,
    pub count: Count,
    pub pitch_data: Option<PitchData>,
    pub is_pitch: bool,
    pub pitch_number: Option<u8>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Count {
    pub balls: u8,
    pub strikes: u8,
    pub outs: u8,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Details {
    pub description: Option<String>,
    pub call: Option<CodeDescription>,
    pub ball_color: Option<String>,
    pub trail_color: Option<String>,
    pub is_in_play: Option<bool>,
    pub is_strike: Option<bool>,
    pub is_ball: Option<bool>,
    #[serde(rename = "type")]
    pub pitch_type: Option<CodeDescription>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct CodeDescription {
    pub code: String,
    pub description: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PitchData {
    pub start_speed: Option<f64>,
    pub end_speed: Option<f64>,
    pub strike_zone_top: Option<f64>,
    pub strike_zone_bottom: Option<f64>,
    pub coordinates: HashMap<String, f64>,
    pub breaks: Option<Breaks>,
    pub zone: Option<u8>,
    pub plate_time: Option<f64>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Breaks {
    pub break_angle: Option<f64>,
    pub break_length: Option<f64>,
    pub break_y: Option<f64>,
    pub spin_rate: Option<u32>,
    pub spin_direction: Option<u32>,
}
