use crate::live::Person;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Boxscore {
    pub teams: Option<Teams>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Teams {
    pub away: Team,
    pub home: Team,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub team: IdName,
    pub team_stats: TeamStats,
    pub players: HashMap<String, Player>,
    pub batters: Vec<u64>,
    pub pitchers: Vec<u64>,
    bench: Vec<u64>,
    bullpen: Vec<u64>,
    pub batting_order: Vec<u64>,
    pub season_stats: Option<TeamStats>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdName {
    pub id: u16,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub person: Person,
    pub position: Position,
    pub stats: TeamStats,
    pub season_stats: TeamStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub name: String,
    pub abbreviation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamStats {
    pub batting: Batting,
    // pitching: TeamStatsPitching,
    // fielding: Fielding,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Batting {
    games_played: Option<u16>,
    pub runs: Option<u16>,
    doubles: Option<u16>,
    triples: Option<u16>,
    pub home_runs: Option<u16>,
    pub strike_outs: Option<u16>,
    pub base_on_balls: Option<u16>,
    pub hits: Option<u16>,
    hit_by_pitch: Option<u16>,
    pub avg: Option<String>,
    pub at_bats: Option<u16>,
    obp: Option<String>,
    slg: Option<String>,
    ops: Option<String>,
    ground_into_double_play: Option<u16>,
    ground_into_triple_play: Option<u16>,
    pub plate_appearances: Option<u16>,
    total_bases: Option<u16>,
    pub rbi: Option<u16>,
    pub left_on_base: Option<u16>,
}

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct TeamStatsPitching {
//     runs: u8,
//     doubles: u8,
//     triples: u8,
//     home_runs: u8,
//     strike_outs: u8,
//     base_on_balls: u8,
//     intentional_walks: u8,
//     hits: u8,
//     hit_by_pitch: u8,
//     at_bats: u8,
//     obp: String,
//     era: String,
//     innings_pitched: String,
//     save_opportunities: u8,
//     earned_runs: u8,
//     whip: String,
//     batters_faced: u8,
//     outs: u8,
//     shutouts: u8,
//     hit_batsmen: u8,
//     rbi: u8,
// }
