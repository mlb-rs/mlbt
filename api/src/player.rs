use crate::live::{PrimaryPosition, Side};
use crate::schedule::IdNameLink;
use crate::stats::Stat;
use serde::Deserialize;

#[derive(Default, Debug, Deserialize)]
pub struct PeopleResponse {
    pub people: Vec<PersonFull>,
}

/// Full player info with hydrated currentTeam and inline stats.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonFull {
    pub id: u64,
    pub full_name: String,
    pub primary_number: Option<String>,
    pub birth_date: Option<String>,
    pub current_age: Option<u8>,
    pub birth_city: Option<String>,
    pub birth_state_province: Option<String>,
    pub birth_country: Option<String>,
    pub height: Option<String>,
    pub weight: Option<u16>,
    pub primary_position: Option<PrimaryPosition>,
    pub bat_side: Option<Side>,
    pub pitch_hand: Option<Side>,
    pub mlb_debut_date: Option<String>,
    pub active: Option<bool>,
    pub draft_year: Option<u16>,
    pub current_team: Option<IdNameLink>,
    pub nick_name: Option<String>,
    pub pronunciation: Option<String>,
    /// Inline stats from hydration. Contains one entry per requested stat type
    /// (season, yearByYear, career, gameLog).
    #[serde(default)]
    pub stats: Vec<Stat>,
    pub drafts: Option<Vec<DraftInfo>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftInfo {
    pub pick_round: String,
    pub pick_number: u16,
    pub round_pick_number: u8,
    pub team: IdNameLink,
    pub is_drafted: bool,
    pub is_pass: bool,
    pub year: String,
}
