use crate::live::Person;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Boxscore {
    pub teams: Option<Teams>,
    pub info: Option<Vec<LabelValue>>,
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
    pub info: Option<Vec<TeamInfo>>,
    pub note: Option<Vec<LabelValue>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdName {
    pub id: u16,
    pub name: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamInfo {
    pub title: String,
    pub field_list: Vec<LabelValue>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelValue {
    pub label: String,
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub person: Person,
    pub position: Position,
    pub batting_order: Option<String>,
    pub stats: TeamStats,
    pub season_stats: TeamStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub name: String,
    pub abbreviation: String,
    #[serde(rename = "type")]
    pub position_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamStats {
    pub batting: Batting,
    pub pitching: PitchingStats,
    // fielding: Fielding,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Batting {
    pub note: Option<String>,
    pub summary: Option<String>,
    games_played: Option<u16>,
    pub runs: Option<u16>,
    doubles: Option<u16>,
    triples: Option<u16>,
    home_runs: Option<u16>,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PitchingStats {
    pub summary: Option<String>,
    pub note: Option<String>,
    pub games_played: Option<i64>,
    pub games_started: Option<i64>,
    pub fly_outs: Option<i64>,
    pub ground_outs: Option<i64>,
    pub air_outs: Option<i64>,
    pub runs: Option<i64>,
    pub doubles: Option<i64>,
    pub triples: Option<i64>,
    pub home_runs: Option<i64>,
    pub strike_outs: Option<i64>,
    pub base_on_balls: Option<i64>,
    pub intentional_walks: Option<i64>,
    pub hits: Option<i64>,
    pub hit_by_pitch: Option<i64>,
    pub at_bats: Option<i64>,
    pub caught_stealing: Option<i64>,
    pub stolen_bases: Option<i64>,
    pub stolen_base_percentage: Option<String>,
    pub number_of_pitches: Option<i64>,
    pub era: Option<String>,
    pub innings_pitched: Option<String>,
    pub wins: Option<i64>,
    pub losses: Option<i64>,
    pub saves: Option<i64>,
    pub save_opportunities: Option<i64>,
    pub holds: Option<i64>,
    pub blown_saves: Option<i64>,
    pub earned_runs: Option<i64>,
    pub batters_faced: Option<i64>,
    pub outs: Option<i64>,
    pub games_pitched: Option<i64>,
    pub complete_games: Option<i64>,
    pub shutouts: Option<i64>,
    pub pitches_thrown: Option<i64>,
    pub balls: Option<i64>,
    pub strikes: Option<i64>,
    pub strike_percentage: Option<String>,
    pub hit_batsmen: Option<i64>,
    pub balks: Option<i64>,
    pub wild_pitches: Option<i64>,
    pub pickoffs: Option<i64>,
    pub rbi: Option<i64>,
    pub games_finished: Option<i64>,
    pub runs_scored_per9: Option<String>,
    pub home_runs_per9: Option<String>,
    pub inherited_runners: Option<i64>,
    pub inherited_runners_scored: Option<i64>,
    pub catchers_interference: Option<i64>,
    pub sac_bunts: Option<i64>,
    pub sac_flies: Option<i64>,
    pub passed_ball: Option<i64>,
    pub pop_outs: Option<i64>,
    pub line_outs: Option<i64>,
}
