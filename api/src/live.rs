use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveResponse {
    pub game_pk: u64,
    pub link: String,
    pub meta_data: MetaData,
    pub game_data: GameData,
    pub live_data: LiveData,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveData {
    pub plays: Plays,
    pub linescore: Linescore,
    pub boxscore: Boxscore,
    // pub leaders: Leaders,
}

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
pub struct PlaysByInning {
    pub start_index: u8,
    pub end_index: u8,
    pub top: Vec<u8>,
    pub bottom: Vec<u8>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Linescore {
    pub current_inning: Option<u8>,
    pub current_inning_ordinal: Option<String>,
    pub inning_state: Option<String>,
    pub inning_half: Option<String>,
    pub is_top_inning: Option<bool>,
    pub scheduled_innings: u8,
    pub innings: Vec<Inning>,
    // pub teams:
    // pub defense:
    pub offense: Offense,
    pub balls: Option<u8>,
    pub strikes: Option<u8>,
    pub outs: Option<u8>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inning {
    pub num: u8,
    pub ordinal_num: String,
    pub home: TeamInningDetail,
    pub away: TeamInningDetail,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamInningDetail {
    pub runs: Option<u8>,
    pub hits: u8,
    pub errors: u8,
    pub left_on_base: u8,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Offense {
    pub on_deck: Option<PlayerIdName>,
    pub in_hole: Option<PlayerIdName>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PlayerIdName {
    pub id: u64,
    #[serde(rename = "fullName")]
    pub full_name: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Boxscore {
    pub teams: Option<BoxscoreTeams>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoxscoreTeams {
    pub away: BoxscoreTeam,
    pub home: BoxscoreTeam,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoxscoreTeam {
    pub team: Team,
    #[serde(rename = "teamStats")]
    pub team_stats: TeamStats,
    pub players: HashMap<String, BoxscorePlayer>,
    pub batters: Vec<u64>,
    pub pitchers: Vec<u64>,
    bench: Vec<u64>,
    bullpen: Vec<u64>,
    #[serde(rename = "battingOrder")]
    pub batting_order: Vec<u64>,
    #[serde(rename = "seasonStats")]
    pub season_stats: Option<TeamStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoxscorePlayer {
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

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaData {
    pub wait: i64,
    pub time_stamp: String,
    pub game_events: Vec<String>,
    pub logical_events: Vec<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameData {
    pub game: Game,
    // pub datetime: Datetime,
    // pub status: Status,
    pub teams: Teams,
    // pub players: Players,
    // pub venue: Venue,
    // pub weather: Weather,
    // pub game_info: GameInfo,
    // pub review: Review,
    // pub flags: Flags,
    // pub alerts: Vec<::serde_json::Value>,
    // pub probable_pitchers: ProbablePitchers,
    // pub official_scorer: OfficialScorer,
    // pub primary_datacaster: PrimaryDatacaster,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub pk: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub double_header: String,
    pub id: String,
    pub gameday_type: String,
    pub tiebreaker: String,
    pub game_number: i64,
    #[serde(rename = "calendarEventID")]
    pub calendar_event_id: String,
    pub season: String,
    pub season_display: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Teams {
    pub away: Team,
    pub home: Team,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: u16,
    pub name: String,
    pub team_name: Option<String>,
    pub short_name: Option<String>,
    pub season: Option<u16>,
    pub team_code: Option<String>,
    pub abbreviation: Option<String>,
    pub location_name: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Person {
    pub id: u64,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub link: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Matchup {
    pub batter: Person,
    pub bat_side: Side,
    pub pitcher: Person,
    pub pitch_hand: Side,
    pub batter_hot_cold_zone_stats: Option<BatterHotColdZoneStats>,
    pub batter_hot_cold_zones: Option<Vec<Zone>>,
    pub post_on_first: Option<Person>,
    pub post_on_second: Option<Person>,
    pub post_on_third: Option<Person>,
}
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Side {
    pub code: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Play {
    pub result: Result,
    pub about: About,
    pub count: Count,
    pub matchup: Matchup,
    #[serde(rename = "playEvents")]
    pub play_events: Vec<PlayEvent>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Result {
    // #[serde(rename = "type")]
    // pub result_type: Option<ResultType>,
    pub event: Option<String>,
    #[serde(rename = "eventType")]
    pub event_type: Option<String>,
    pub description: Option<String>,
    pub rbi: Option<u8>,
    #[serde(rename = "awayScore")]
    pub away_score: Option<u8>,
    #[serde(rename = "homeScore")]
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
pub struct BatterHotColdZoneStats {
    pub stats: Vec<StatElement>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct StatElement {
    pub splits: Vec<Split>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Split {
    pub stat: SplitStat,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SplitStat {
    pub name: String,
    pub zones: Vec<Zone>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zone {
    pub zone: String,
    pub color: String, // this is what I want: "rgba(255, 255, 255, 0.55)" -> need to convert it to a color
    pub value: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Count {
    pub balls: u8,
    pub strikes: u8,
    pub outs: u8,
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
