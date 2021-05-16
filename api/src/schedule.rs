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
pub struct IdNameLink {
    pub id: u16,
    pub name: String,
    pub link: String,
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
pub struct Game {
    pub game_pk: u64,
    pub link: String,
    // pub game_type: Option<GameType>,
    pub season: String,
    pub game_date: String,
    pub official_date: String,
    pub status: Status,
    pub teams: Teams,
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
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Content {
    pub link: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub abstract_game_state: Option<AbstractGameState>,
    pub coded_game_state: Option<CodedGameState>,
    pub detailed_state: Option<DetailedState>,
    pub status_code: Option<StatusCode>,
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
}

#[derive(Serialize, Deserialize)]
pub enum DayNight {
    #[serde(rename = "day")]
    Day,
    #[serde(rename = "night")]
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

#[derive(Debug, Serialize, Deserialize)]
pub enum AbstractGameState {
    Final,
    Live,
    Preview,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CodedGameState {
    // pre game
    P,
    // in progress
    I,
    // final
    F,
    // over
    O,
    // postponed
    D,
    // suspended
    U,
    // scheduled
    S,
    // manager challenge
    M,
    // unknown
    N,
}

#[derive(Debug, strum_macros::Display, Serialize, Deserialize)]
pub enum DetailedState {
    #[serde(rename = "Pre-Game")]
    PreGame,
    Warmup,
    #[serde(rename = "In Progress")]
    InProgress,
    #[serde(rename = "Game Over")]
    GameOver,
    Final,
    Postponed,
    Suspended,
    Scheduled,
    Delayed,
    #[serde(rename = "Delayed Start")]
    DelayedStart,
    #[serde(alias = "Manager Challenge", alias = "Manager challenge")]
    ManagerChallenge,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StatusCode {
    // pre game
    P,
    // warmup
    #[serde(rename = "PW")]
    Pw,
    // delayed start
    #[serde(rename = "PR")]
    Pr,
    // in progress
    I,
    // final
    F,
    // game over
    O,
    // suspended
    #[serde(rename = "UI")]
    Ui,
    // unknown
    #[serde(rename = "DR")]
    Dr,
    // postponed
    #[serde(rename = "DI")]
    Di,
    // scheduled
    S,
    // unknown
    #[serde(rename = "IR")]
    Ir,
    // manager challenge
    #[serde(rename = "MA")]
    Ma,
}

// #[derive(Serialize, Deserialize)]
// pub enum GameType {
//     R,
// }

// #[derive(Serialize, Deserialize)]
// pub enum GamedayType {
//     P,
// }
//
// #[derive(Serialize, Deserialize)]
// pub enum IfNecessaryDescription {
//     #[serde(rename = "Normal Game")]
//     NormalGame,
// }
//
// #[derive(Serialize, Deserialize)]
// pub enum RecordSource {
//     S,
// }
//
// #[derive(Serialize, Deserialize)]
// pub enum SeriesDescription {
//     #[serde(rename = "Regular Season")]
//     RegularSeason,
// }
