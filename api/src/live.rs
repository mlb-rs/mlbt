use crate::boxscore::Boxscore;
use crate::plays::Plays;

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
