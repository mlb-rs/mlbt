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

/// A team's position in the postseason race. The API sends the numbers as strings carrying `-`
/// and `E` sentinels, which become `None` plus the two flags here.
#[derive(Debug, Default, Clone)]
pub struct ClinchStatus {
    pub indicator: Option<ClinchIndicator>,
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
    fn parse_number(value: Option<&str>) -> Option<u8> {
        value.and_then(|v| v.parse().ok())
    }

    fn from_team_record(team: &TeamRecord) -> Self {
        Self::from_parts(
            team.clinch_indicator,
            team.elimination_number.as_deref(),
            team.wild_card_elimination_number.as_deref(),
        )
    }

    fn from_parts(
        indicator: Option<ClinchIndicator>,
        elimination: Option<&str>,
        wild_card_elimination: Option<&str>,
    ) -> Self {
        let division_eliminated = elimination == Some("E");
        let eliminated = division_eliminated && wild_card_elimination == Some("E");

        Self {
            indicator,
            elimination_number: Self::parse_number(elimination),
            wild_card_elimination_number: Self::parse_number(wild_card_elimination),
            division_eliminated,
            eliminated,
        }
    }

    /// The letter shown next to a team name once it clinches. Elimination is not a marker, it
    /// shows up as `E` in the elimination number column instead.
    pub fn marker(&self) -> Option<char> {
        match self.indicator? {
            ClinchIndicator::Z => Some('z'),
            ClinchIndicator::Y => Some('y'),
            ClinchIndicator::X => Some('x'),
            ClinchIndicator::W => Some('w'),
            ClinchIndicator::Unknown => None,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clinch_status_reads_the_api_sentinels() {
        // a division leader has nothing to be eliminated from, so the API sends `-`
        let leader = ClinchStatus::from_parts(None, Some("-"), Some("-"));
        assert_eq!(leader.elimination_number, None);
        assert!(!leader.division_eliminated);

        let chaser = ClinchStatus::from_parts(None, Some("26"), Some("30"));
        assert_eq!(chaser.elimination_number, Some(26));
        assert_eq!(chaser.wild_card_elimination_number, Some(30));

        // `-` and `E` are sentinels, not numbers
        let out = ClinchStatus::from_parts(None, Some("E"), Some("E"));
        assert_eq!(out.elimination_number, None);
        assert!(out.division_eliminated);
        assert!(out.eliminated);
    }

    #[test]
    fn eliminated_requires_being_out_of_the_wild_card_too() {
        // clinched a wild card, so out of the division race but playing in October
        let wild_card = ClinchStatus::from_parts(Some(ClinchIndicator::W), Some("E"), Some("-"));
        assert!(wild_card.division_eliminated);
        assert!(!wild_card.eliminated);
        assert_eq!(wild_card.marker(), Some('w'));

        // still chasing a wild card
        let contender = ClinchStatus::from_parts(None, Some("E"), Some("5"));
        assert!(contender.division_eliminated);
        assert!(!contender.eliminated);

        // out of both races
        let out = ClinchStatus::from_parts(None, Some("E"), Some("E"));
        assert!(out.eliminated);
        assert_eq!(out.marker(), None);
    }

    #[test]
    fn marker_comes_from_the_clinch_indicator() {
        let marker =
            |indicator| ClinchStatus::from_parts(Some(indicator), Some("-"), Some("-")).marker();

        assert_eq!(marker(ClinchIndicator::Z), Some('z'));
        assert_eq!(marker(ClinchIndicator::Y), Some('y'));
        assert_eq!(marker(ClinchIndicator::X), Some('x'));
        assert_eq!(marker(ClinchIndicator::W), Some('w'));

        assert_eq!(marker(ClinchIndicator::Unknown), None);

        // still in the race
        assert_eq!(ClinchStatus::default().marker(), None);
    }
}
