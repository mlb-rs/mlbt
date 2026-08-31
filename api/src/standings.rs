use crate::schedule::IdNameLink;
use chrono::{DateTime, Utc};
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
    pub division: Option<IdLink>,
    pub last_updated: DateTime<Utc>,
    pub team_records: Vec<TeamRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdLink {
    pub id: u8,
    pub link: String,
}

/// Postseason clinch status, from the API's `clinchIndicator`.
/// Absent until a team clinches, and cleared once the regular season ends.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClinchIndicator {
    /// clinched the best record
    Z,
    /// clinched the division
    Y,
    /// clinched a playoff berth
    X,
    /// clinched a wild card
    W,
    /// a value the API added that isn't modeled here
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamRecord {
    pub team: IdNameLink,
    pub season: String,
    pub streak: Option<Streak>,
    pub clinch_indicator: Option<ClinchIndicator>,
    pub division_rank: Option<String>,
    pub league_rank: String,
    pub sport_rank: Option<String>,
    pub games_played: u8,
    pub games_back: String,
    pub wild_card_games_back: String,
    pub league_games_back: String,
    pub sport_games_back: String,
    pub division_games_back: String,
    pub conference_games_back: String,
    pub league_record: RecordElement,
    pub last_updated: DateTime<Utc>,
    pub records: Records,
    pub runs_allowed: u16,
    pub runs_scored: u16,
    pub division_champ: Option<bool>,
    pub division_leader: bool,
    pub has_wildcard: Option<bool>,
    pub clinched: Option<bool>,
    pub elimination_number: Option<String>,
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
    pub division_records: Option<Vec<RecordElement>>,
    pub overall_records: Vec<RecordElement>,
    pub league_records: Vec<RecordElement>,
    pub expected_records: Option<Vec<RecordElement>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Streak {
    pub streak_type: Option<String>,
    pub streak_number: Option<u8>,
    pub streak_code: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clinch_indicator_maps_api_values() {
        let parse = |s: &str| serde_json::from_str::<Option<ClinchIndicator>>(s).unwrap();

        assert_eq!(parse("\"z\""), Some(ClinchIndicator::Z));
        assert_eq!(parse("\"y\""), Some(ClinchIndicator::Y));
        assert_eq!(parse("\"x\""), Some(ClinchIndicator::X));
        assert_eq!(parse("\"w\""), Some(ClinchIndicator::W));

        // a letter the API adds later must not fail the whole standings response
        assert_eq!(parse("\"e\""), Some(ClinchIndicator::Unknown));
        assert_eq!(parse("null"), None);
    }
}
