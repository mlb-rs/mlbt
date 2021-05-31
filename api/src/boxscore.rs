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
pub struct Team {
    pub team: IdName,
    #[serde(rename = "teamStats")]
    pub team_stats: TeamStats,
    pub players: HashMap<String, Player>,
    pub batters: Vec<u64>,
    pub pitchers: Vec<u64>,
    bench: Vec<u64>,
    bullpen: Vec<u64>,
    #[serde(rename = "battingOrder")]
    pub batting_order: Vec<u64>,
    #[serde(rename = "seasonStats")]
    pub season_stats: Option<TeamStats>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdName {
    pub id: u16,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub person: Person,
    pub position: Position,
    pub stats: TeamStats,
    #[serde(rename = "seasonStats")]
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
    games_played: Option<u8>,
    pub runs: Option<u8>,
    doubles: Option<u8>,
    triples: Option<u8>,
    home_runs: Option<u8>,
    pub strike_outs: Option<u8>,
    pub base_on_balls: Option<u8>,
    pub hits: Option<u8>,
    hit_by_pitch: Option<u8>,
    pub avg: Option<String>,
    pub at_bats: Option<u8>,
    obp: Option<String>,
    slg: Option<String>,
    ops: Option<String>,
    ground_into_double_play: Option<u8>,
    ground_into_triple_play: Option<u8>,
    pub plate_appearances: Option<u8>,
    total_bases: Option<u8>,
    pub rbi: Option<u8>,
    pub left_on_base: Option<u8>,
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
