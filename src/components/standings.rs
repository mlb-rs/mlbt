use crate::components::constants::DIVISIONS;
use core::option::Option::{None, Some};
use mlb_api::standings::{StandingsResponse, TeamRecord};
use tui::widgets::TableState;

/// Stores the state for rendering the standings. The `standings` field is a nested Vec to make
/// displaying by division easier.
pub struct StandingsState {
    pub state: TableState,
    pub standings: Vec<Division>,
    pub team_ids: Vec<u16>,
}

/// Groups teams into their divisions.
pub struct Division {
    pub name: String,
    id: u8,
    pub standings: Vec<Standing>,
}

/// Standing information per team.
#[derive(Debug, Default)]
pub struct Standing {
    pub team_name: String,
    pub team_id: u16,
    pub wins: u8,
    pub losses: u8,
    pub winning_percentage: String,
    pub games_back: String,
    pub wild_card_games_back: String,
    pub streak: String,
}

impl Default for StandingsState {
    fn default() -> Self {
        let mut ss = StandingsState {
            state: TableState::default(),
            standings: Division::create_divisions(),
            team_ids: vec![200, 201, 202, 203, 204, 205],
        };
        ss.state.select(Some(0));
        ss
    }
}

impl StandingsState {
    pub fn update(&mut self, standings: &StandingsResponse) {
        self.standings = Division::create_table(standings);
        self.team_ids = self.generate_ids();
    }

    fn generate_ids(&mut self) -> Vec<u16> {
        let mut ids = Vec::with_capacity(36); // 30 teams, 6 divisions
        for division in &self.standings {
            ids.push(division.id as u16);
            for team in &division.standings {
                ids.push(team.team_id);
            }
        }
        ids
    }

    pub fn get_selected(&self) -> u16 {
        if let Some(s) = self.team_ids.get(
            self.state
                .selected()
                .expect("there is always a selected standing"),
        ) {
            *s
        } else {
            0
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.team_ids.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.team_ids.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl Division {
    /// Generate only the division names.
    fn create_divisions() -> Vec<Division> {
        (200..206)
            .map(|id| Division {
                name: DIVISIONS.get(&id).unwrap().to_string(),
                id,
                standings: vec![],
            })
            .collect()
    }

    /// Generate the standings data to be used to render a table widget.
    fn create_table(standings: &StandingsResponse) -> Vec<Division> {
        let mut s: Vec<Division> = standings
            .records
            .iter()
            .map(|r| Division {
                name: DIVISIONS.get(&r.division.id).unwrap().to_string(),
                id: r.division.id,
                standings: r
                    .team_records
                    .iter()
                    .map(Standing::from_team_record)
                    .collect(),
            })
            .collect();
        // ensure display order is the same
        s.sort_by(|a, b| a.id.cmp(&b.id));
        s
    }
}

impl Standing {
    fn from_team_record(team: &TeamRecord) -> Self {
        let streak = if let Some(s) = team.streak.as_ref() {
            s.streak_code.clone()
        } else {
            "-".to_string()
        };
        Standing {
            team_name: team.team.name.clone(),
            team_id: team.team.id,
            wins: team.wins,
            losses: team.losses,
            winning_percentage: team.winning_percentage.clone(),
            games_back: team.games_back.clone(),
            wild_card_games_back: team.wild_card_games_back.clone(),
            streak,
        }
    }

    pub fn to_cells(&self) -> Vec<String> {
        vec![
            self.team_name.clone(),
            self.wins.to_string(),
            self.losses.to_string(),
            self.winning_percentage.clone(),
            self.games_back.clone(),
            self.wild_card_games_back.clone(),
            self.streak.clone(),
        ]
    }
}
