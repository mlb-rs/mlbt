use crate::stats::DisplayName;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleResponse {
    pub copyright: Option<String>,
    pub total_items: Option<u8>,
    pub total_events: Option<u8>,
    pub total_games: u8,
    pub total_games_in_progress: Option<u8>,
    pub dates: Vec<Dates>, // these are the actual games
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct LeagueRecord {
    pub wins: u8,
    pub losses: u8,
    pub pct: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdNameLink {
    pub id: u16,
    pub name: String,
    pub link: String,
    /// Present when the team is a minor-league affiliate, points to the MLB parent org.
    pub parent_org_id: Option<u16>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dates {
    pub date: Option<String>,
    pub total_items: Option<u8>,
    pub total_events: Option<u8>,
    pub total_games: Option<u8>,
    pub total_games_in_progress: Option<u8>,
    pub games: Option<Vec<Game>>,
    pub events: Option<Vec<Option<serde_json::Value>>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleLinescore {
    pub current_inning: Option<i64>,
    pub current_inning_ordinal: Option<String>,
    pub inning_state: Option<String>,
    pub inning_half: Option<String>,
    pub is_top_inning: Option<bool>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub game_pk: u64,
    pub link: String,
    // pub game_type: Option<GameType>,
    pub season: String,
    pub game_date: String,
    pub official_date: String,
    pub status: Status,
    pub teams: Teams,
    /// Only present if `hydrate=linescore` is used.
    pub linescore: Option<ScheduleLinescore>,
    pub venue: Option<IdNameLink>,
    pub content: Option<Content>,
    pub is_tie: Option<bool>,
    pub game_number: Option<u64>,
    // pub public_facing: Option<bool>,
    // pub double_header: Option<DoubleHeader>,
    // pub gameday_type: Option<GamedayType>,
    // pub tiebreaker: Option<DoubleHeader>,
    pub calendar_event_id: Option<String>,
    // pub season_display: Option<String>,
    // pub day_night: Option<DayNight>,
    // pub scheduled_innings: Option<i64>,
    // pub reverse_home_away_status: Option<bool>,
    // pub inning_break_length: Option<i64>,
    // pub games_in_series: Option<i64>,
    // pub series_game_number: Option<i64>,
    // pub series_description: Option<SeriesDescription>,
    // pub record_source: Option<RecordSource>,
    // pub if_necessary: Option<DoubleHeader>,
    // pub if_necessary_description: Option<IfNecessaryDescription>,
    // pub rescheduled_from: Option<String>,
    // pub description: Option<String>,
    // pub resume_date: Option<String>,
    // pub reschedule_date: Option<String>,
    pub decisions: Option<Decisions>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Content {
    pub link: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub abstract_game_state: Option<AbstractGameState>,
    pub coded_game_state: Option<String>,
    pub detailed_state: Option<String>,
    pub status_code: Option<String>,
    #[serde(rename = "startTimeTBD")]
    pub start_time_tbd: Option<bool>,
    pub abstract_game_code: Option<AbstractGameCode>,
    pub reason: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Teams {
    pub away: TeamInfo,
    pub home: TeamInfo,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamInfo {
    pub league_record: Option<LeagueRecord>,
    pub score: Option<u8>,
    pub team: IdNameLink,
    pub is_winner: Option<bool>,
    pub split_squad: Option<bool>,
    pub series_number: Option<u8>,
    pub probable_pitcher: Option<ProbablePitcher>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DayNight {
    Day,
    Night,
}

#[derive(Serialize, Deserialize)]
pub enum DoubleHeader {
    N,
    Y,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AbstractGameCode {
    // pre game
    P,
    // live
    L,
    // final
    F,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum AbstractGameState {
    Final,
    Live,
    Preview,
    Other,
}

/// Only present if `hydrate=probablePitcher` is used.
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbablePitcher {
    pub full_name: String,
    #[serde(default)]
    pub stats: Vec<StatEntry>,
}

/// Only present if `hydrate=stats` is used.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct StatEntry {
    #[serde(rename = "type")]
    pub stat_type: Option<DisplayName>,
    pub group: Option<DisplayName>,
    pub stats: Option<PitcherStats>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PitcherStats {
    pub note: Option<String>,
    pub summary: Option<String>,
    pub strike_outs: Option<u16>,
    pub base_on_balls: Option<u16>,
    pub era: Option<String>,
    pub innings_pitched: Option<String>,
    pub wins: Option<u8>,
    pub losses: Option<u8>,
}

/// Only present if `hydrate=decisions` is used, and only present for Final games.
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Decisions {
    pub winner: DecisionPitcher,
    pub loser: DecisionPitcher,
    pub save: Option<DecisionPitcher>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionPitcher {
    pub full_name: String,
    #[serde(default)]
    pub stats: Vec<StatEntry>,
}
