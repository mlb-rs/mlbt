use crate::components::constants::{DIVISION_ORDERS, DIVISIONS, lookup_team};
use mlbt_api::standings::{ClinchIndicator, RecordElement, StandingsResponse, TeamRecord};
use std::string::ToString;

/// Groups teams into their divisions.
pub struct Division {
    pub name: String,
    pub id: u16,
    pub standings: Vec<Standing>,
}

#[derive(Debug, Clone, Copy)]
pub struct Team {
    pub id: u16,
    pub division_id: u16,
    /// Full name, e.g. "Chicago Cubs"
    pub name: &'static str,
    /// Short name, e.g. "Cubs"
    pub team_name: &'static str,
    /// All caps abbreviation, e.g. "CHC"
    pub abbreviation: &'static str,
}

/// A team's position in the postseason race. The API reports magic and elimination numbers as
/// strings that carry sentinels (`-` when not applicable, `E` once eliminated), so they are parsed
/// to numbers here and the sentinels become `None` or `eliminated`.
#[derive(Debug, Default, Clone)]
pub struct ClinchStatus {
    /// Set once a team clinches and cleared after the regular season ends.
    pub indicator: Option<ClinchIndicator>,
    /// The division magic number, only set for the division leader.
    pub magic_number: Option<u8>,
    /// The division elimination number, only set for teams not leading their division.
    pub elimination_number: Option<u8>,
    /// The wild card elimination number.
    pub wild_card_elimination_number: Option<u8>,
    /// Out of the division race but not playoffs (wildcard).
    pub division_eliminated: bool,
    /// Out of the postseason entirely, which needs both elimination numbers to be `E`.
    pub eliminated: bool,
}

/// Standing information per team.
#[derive(Debug, Default, Clone)]
pub struct Standing {
    pub team: Team,
    pub wins: u8,
    pub losses: u8,
    pub winning_percentage: String,
    pub games_back: String,
    pub wild_card_games_back: String,
    /// The API only ranks teams in the wild card race, so division leaders are `None`.
    pub wild_card_rank: Option<u8>,
    pub last_10: String,
    pub streak: String,
    pub runs_scored: u16,
    pub runs_allowed: u16,
    pub run_differential: i16,
    pub xwl: String,
    pub home: String,
    pub away: String,
    pub clinch: ClinchStatus,
}

impl Default for Team {
    fn default() -> Self {
        Self {
            id: 0,
            division_id: 0,
            name: "unknown",
            team_name: "unknown",
            abbreviation: "UNK",
        }
    }
}

impl Team {
    /// Create a team from the schedule API response data.
    /// Uses `Box::leak` to promote strings to `&'static str`.
    pub fn from_schedule(team: &mlbt_api::schedule::IdNameLink) -> Self {
        let leaked: &'static str = Box::leak(team.name.clone().into_boxed_str());
        Self {
            id: team.id,
            name: leaked,
            team_name: leaked,
            abbreviation: leaked,
            ..Self::default()
        }
    }

    /// Create a team from the live game API response data.
    /// Uses `Box::leak` to promote strings to `&'static str`.
    pub fn from_live(team: &mlbt_api::live::Team) -> Self {
        Self {
            id: team.id,
            name: Box::leak(team.name.clone().into_boxed_str()),
            team_name: Box::leak(team.team_name.clone().into_boxed_str()),
            abbreviation: Box::leak(team.abbreviation.clone().into_boxed_str()),
            ..Self::default()
        }
    }
}

impl Division {
    /// Generate only the division names.
    pub fn create_divisions() -> Vec<Division> {
        (200..206)
            .map(|id| Division {
                name: DIVISIONS[&id].to_string(),
                id,
                standings: vec![],
            })
            .collect()
    }

    /// Generate the standings data to be used to render a table widget.
    pub fn create_table(
        standings: &StandingsResponse,
        favorite_team: Option<Team>,
    ) -> Vec<Division> {
        let mut s: Vec<Division> = standings
            .records
            .iter()
            .map(|r| {
                // Pre-1969 seasons have no divisions, fall back to league.
                let group_id = r
                    .division
                    .as_ref()
                    .map(|d| d.id as u16)
                    .unwrap_or(r.league.id as u16);
                let group_name = DIVISIONS.get(&group_id).unwrap_or(&"Unknown").to_string();
                Division {
                    name: group_name,
                    id: group_id,
                    standings: r
                        .team_records
                        .iter()
                        .map(Standing::from_team_record)
                        .collect(),
                }
            })
            .collect();

        Self::sort_by_favorite(&mut s, favorite_team);
        s
    }

    pub fn sort_by_favorite(divisions: &mut [Division], favorite_team: Option<Team>) {
        if let Some(team) = favorite_team
            && let Some(order) = DIVISION_ORDERS.get(&team.division_id)
        {
            divisions.sort_by_key(|standing| {
                order
                    .iter()
                    .position(|&x| x == standing.id)
                    .unwrap_or(usize::MAX)
            });
            return;
        }

        // ensure display order is the same when there is no favorite team ordering
        divisions.sort_by_key(|a| a.id);
    }
}

impl ClinchStatus {
    /// The API reports these as strings with `-` and `E` sentinels, so anything that isn't a
    /// number becomes `None`.
    fn parse_number(value: Option<&String>) -> Option<u8> {
        value.and_then(|v| v.parse().ok())
    }

    fn from_team_record(team: &TeamRecord) -> Self {
        // eliminationNumber alone is the division race, so a wild card team reads as `E` there
        // while still being in the postseason
        let division_eliminated = team.elimination_number.as_deref() == Some("E");
        let eliminated =
            division_eliminated && team.wild_card_elimination_number.as_deref() == Some("E");

        Self {
            indicator: team.clinch_indicator,
            magic_number: Self::parse_number(team.magic_number.as_ref()),
            elimination_number: Self::parse_number(team.elimination_number.as_ref()),
            wild_card_elimination_number: Self::parse_number(
                team.wild_card_elimination_number.as_ref(),
            ),
            division_eliminated,
            eliminated,
        }
    }

    /// The letter shown next to a team name, or `None` while a team is still in the race.
    pub fn marker(&self) -> Option<char> {
        match self.indicator {
            Some(ClinchIndicator::Z) => Some('z'),
            Some(ClinchIndicator::Y) => Some('y'),
            Some(ClinchIndicator::X) => Some('x'),
            Some(ClinchIndicator::W) => Some('w'),
            // a letter the API added that this version doesn't know how to label
            Some(ClinchIndicator::Unknown) => None,
            None if self.eliminated => Some('e'),
            None => None,
        }
    }

    /// The magic or elimination number to display. The API only ever sets one of the two, so
    /// whichever is present is the number for this team's division race.
    pub fn race_number(&self) -> Option<u8> {
        self.magic_number.or(self.elimination_number)
    }
}

impl Standing {
    fn find_record(records: &[RecordElement], record_type: &str) -> String {
        records
            .iter()
            .find(|r| r.record_type.as_deref() == Some(record_type))
            .map(|r| format!("{}-{}", r.wins, r.losses))
            .unwrap_or_else(|| "-".to_string())
    }

    fn from_team_record(team: &TeamRecord) -> Self {
        let streak = team
            .streak
            .as_ref()
            .map(|s| s.streak_code.clone())
            .unwrap_or_else(|| "-".to_string());
        let last_10 = Self::find_record(&team.records.split_records, "lastTen");
        let home = Self::find_record(&team.records.overall_records, "home");
        let away = Self::find_record(&team.records.overall_records, "away");
        let xwl = team
            .records
            .expected_records
            .as_ref()
            .map(|records| Self::find_record(records, "xWinLoss"))
            .unwrap_or_else(|| "-".to_string());

        Standing {
            team: lookup_team(&team.team.name),
            wins: team.wins,
            losses: team.losses,
            winning_percentage: team.winning_percentage.clone(),
            games_back: team.games_back.clone(),
            wild_card_games_back: team.wild_card_games_back.clone(),
            wild_card_rank: team.wild_card_rank.as_deref().and_then(|r| r.parse().ok()),
            last_10,
            streak,
            runs_scored: team.runs_scored,
            runs_allowed: team.runs_allowed,
            run_differential: team.run_differential,
            xwl,
            home,
            away,
            clinch: ClinchStatus::from_team_record(team),
        }
    }
}
