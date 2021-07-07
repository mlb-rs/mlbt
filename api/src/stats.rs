use crate::schedule::IdNameLink;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct StatResponse {
    stats: Vec<Stat>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stat {
    #[serde(rename = "type")]
    stat_type: DisplayName,
    group: DisplayName,
    total_splits: u16,
    splits: Vec<Split>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayName {
    display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Split {
    season: String,
    stat: PitchingStat,
    team: IdNameLink,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PitchingStat {
    games_played: i64,
    games_started: i64,
    ground_outs: i64,
    air_outs: i64,
    runs: i64,
    doubles: i64,
    triples: i64,
    home_runs: i64,
    strike_outs: i64,
    base_on_balls: i64,
    intentional_walks: i64,
    hits: i64,
    hit_by_pitch: i64,
    avg: String,
    at_bats: i64,
    obp: String,
    slg: String,
    ops: String,
    caught_stealing: i64,
    stolen_bases: i64,
    stolen_base_percentage: String,
    ground_into_double_play: i64,
    number_of_pitches: i64,
    era: String,
    innings_pitched: String,
    wins: i64,
    losses: i64,
    saves: i64,
    save_opportunities: i64,
    holds: i64,
    blown_saves: i64,
    earned_runs: i64,
    whip: String,
    batters_faced: i64,
    outs: i64,
    games_pitched: i64,
    complete_games: i64,
    shutouts: i64,
    strikes: i64,
    strike_percentage: String,
    hit_batsmen: i64,
    balks: i64,
    wild_pitches: i64,
    pickoffs: i64,
    total_bases: i64,
    ground_outs_to_airouts: String,
    win_percentage: String,
    pitches_per_inning: String,
    games_finished: i64,
    strikeout_walk_ratio: String,
    strikeouts_per9_inn: String,
    walks_per9_inn: String,
    hits_per9_inn: String,
    runs_scored_per9: String,
    home_runs_per9: String,
    catchers_interference: i64,
    sac_bunts: i64,
    sac_flies: i64,
}
