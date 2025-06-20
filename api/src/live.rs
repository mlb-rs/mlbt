use crate::boxscore::Boxscore;
use crate::plays::Plays;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
    pub teams: Teams,
    pub players: HashMap<String, FullPlayer>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveData {
    pub plays: Plays,
    pub linescore: Linescore,
    pub boxscore: Boxscore,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Linescore {
    pub current_inning: Option<u8>,
    pub current_inning_ordinal: Option<String>,
    pub inning_state: Option<String>,
    pub inning_half: Option<String>,
    pub is_top_inning: Option<bool>,
    pub scheduled_innings: Option<u8>,
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
    pub team_name: String,
    pub short_name: String,
    pub season: u16,
    pub team_code: String,
    pub abbreviation: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Person {
    pub id: u64,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub link: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Side {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrimaryPosition {
    pub code: String,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub abbreviation: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullPlayer {
    pub id: u64,
    pub full_name: String,
    pub link: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub primary_number: Option<String>,
    pub birth_date: Option<String>,
    pub current_age: Option<i64>,
    pub birth_city: Option<String>,
    pub birth_state_province: Option<String>,
    pub birth_country: Option<String>,
    pub height: Option<String>,
    pub weight: Option<u16>,
    pub active: Option<bool>,
    pub primary_position: Option<PrimaryPosition>,
    pub use_name: Option<String>,
    pub use_last_name: Option<String>,
    pub middle_name: Option<String>,
    pub boxscore_name: Option<String>,
    pub gender: Option<String>,
    pub is_player: Option<bool>,
    pub is_verified: Option<bool>,
    pub draft_year: Option<i64>,
    pub mlb_debut_date: Option<String>,
    pub bat_side: Option<Side>,
    pub pitch_hand: Option<Side>,
    pub name_first_last: Option<String>,
    pub name_slug: Option<String>,
    pub first_last_name: Option<String>,
    pub last_first_name: Option<String>,
    pub last_init_name: Option<String>,
    pub init_last_name: String,
    #[serde(rename = "fullFMLName")]
    pub full_fmlname: Option<String>,
    #[serde(rename = "fullLFMName")]
    pub full_lfmname: Option<String>,
    pub strike_zone_top: Option<f64>,
    pub strike_zone_bottom: Option<f64>,
}
