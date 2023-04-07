use crate::schedule::IdNameLink;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct StandingsResponse {
    pub copyright: String,
    pub records: Vec<Record>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub standings_type: String,
    pub league: IdLink,
    pub division: IdLink,
    pub last_updated: String,
    pub team_records: Vec<TeamRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdLink {
    pub id: u8,
    pub link: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamRecord {
    pub team: IdNameLink,
    pub season: String,
    pub streak: Streak,
    pub division_rank: String,
    pub league_rank: String,
    pub sport_rank: String,
    pub games_played: u8,
    pub games_back: String,
    pub wild_card_games_back: String,
    pub league_games_back: String,
    pub sport_games_back: String,
    pub division_games_back: String,
    pub conference_games_back: String,
    pub league_record: RecordElement,
    pub last_updated: String,
    pub records: Records,
    pub runs_allowed: u16,
    pub runs_scored: u16,
    pub division_champ: bool,
    pub division_leader: bool,
    pub has_wildcard: bool,
    pub clinched: bool,
    pub elimination_number: String,
    pub magic_number: Option<String>,
    pub wins: u8,
    pub losses: u8,
    pub run_differential: i16,
    pub winning_percentage: String,
    pub wild_card_rank: Option<String>,
    pub wild_card_leader: Option<bool>,
    pub wild_card_elimination_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordElement {
    pub wins: u8,
    pub losses: u8,
    pub pct: String,
    pub division: Option<IdNameLink>,
    #[serde(rename = "type")]
    pub record_type: Option<String>,
    pub league: Option<IdNameLink>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Records {
    pub split_records: Vec<RecordElement>,
    pub division_records: Vec<RecordElement>,
    pub overall_records: Vec<RecordElement>,
    pub league_records: Vec<RecordElement>,
    pub expected_records: Vec<RecordElement>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Streak {
    pub streak_type: String,
    pub streak_number: u8,
    pub streak_code: String,
}
