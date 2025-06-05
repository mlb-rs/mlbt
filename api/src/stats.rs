use crate::schedule::IdNameLink;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct StatsResponse {
    pub stats: Vec<Stat>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stat {
    #[serde(rename = "type")]
    pub stat_type: DisplayName,
    pub group: DisplayName,
    pub total_splits: u16,
    pub splits: Vec<Split>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayName {
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Split {
    season: Option<String>,
    pub stat: StatSplit,
    pub team: IdNameLink,
    pub player: Option<Player>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub id: u64,
    pub full_name: String,
    pub first_name: String,
    pub last_name: String,
}

/// StatSplit stores the two options for deserializing a Split.
/// It uses the `untagged` enum representation to determine which one.
/// https://serde.rs/enum-representations.html#untagged
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StatSplit {
    Pitching(Box<PitchingStat>),
    Hitting(Box<HittingStat>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PitchingStat {
    pub wins: u16,
    pub losses: u16,
    pub era: String,
    pub games_played: u16,
    pub games_started: u16,
    pub complete_games: u16,
    pub shutouts: u16,
    pub saves: u16,
    pub save_opportunities: u16,
    ground_outs: u16,
    air_outs: u16,
    pub innings_pitched: String,
    pub hits: u16,
    pub runs: u16,
    pub earned_runs: u16,
    doubles: i64,
    triples: i64,
    pub home_runs: u16,
    pub hit_batsmen: u16,
    pub base_on_balls: u16,
    pub strike_outs: u16,
    intentional_walks: u16,
    hit_by_pitch: u16,
    pub whip: String,
    pub avg: String,
    at_bats: u16,
    obp: String,
    slg: String,
    ops: String,
    caught_stealing: u16,
    stolen_bases: u16,
    stolen_base_percentage: String,
    ground_into_double_play: u16,
    number_of_pitches: u16,
    holds: u16,
    blown_saves: u16,
    batters_faced: u16,
    outs: u16,
    games_pitched: u16,
    strikes: u32,
    strike_percentage: String,
    balks: u16,
    wild_pitches: u16,
    pickoffs: Option<u16>,
    total_bases: u16,
    ground_outs_to_airouts: String,
    win_percentage: String,
    pitches_per_inning: String,
    games_finished: u16,
    strikeout_walk_ratio: String,
    strikeouts_per9_inn: String,
    walks_per9_inn: String,
    hits_per9_inn: String,
    runs_scored_per9: String,
    home_runs_per9: String,
    catchers_interference: Option<u16>,
    sac_bunts: u16,
    sac_flies: u16,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HittingStat {
    pub games_played: u16,
    pub ground_outs: u16,
    pub air_outs: u16,
    pub runs: u16,
    pub doubles: u16,
    pub triples: u16,
    pub home_runs: u16,
    pub strike_outs: u16,
    pub base_on_balls: u16,
    pub intentional_walks: u16,
    pub hits: u16,
    pub hit_by_pitch: u16,
    pub avg: String,
    pub at_bats: u16,
    pub obp: String,
    pub slg: String,
    pub ops: String,
    pub caught_stealing: u16,
    pub stolen_bases: u16,
    pub stolen_base_percentage: String,
    pub ground_into_double_play: u16,
    pub number_of_pitches: u16,
    pub plate_appearances: u16,
    pub total_bases: u16,
    pub rbi: u16,
    pub left_on_base: u16,
    pub sac_bunts: u16,
    pub sac_flies: u16,
    pub babip: String,
    pub ground_outs_to_airouts: String,
    pub catchers_interference: Option<u16>,
    pub at_bats_per_home_run: String,
}
