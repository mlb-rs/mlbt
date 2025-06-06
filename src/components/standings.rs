use crate::components::constants::{DIVISION_ORDERS, DIVISIONS};
use crate::components::date_selector::DateSelector;
use chrono::NaiveDate;
use core::option::Option::Some;
use mlb_api::standings::{StandingsResponse, TeamRecord};
use std::collections::HashSet;
use tui::widgets::TableState;

/// Stores the state for rendering the standings. The `standings` field is a nested Vec to make
/// displaying by division easier.
pub struct StandingsState {
    pub state: TableState,
    pub favorite_team: Option<Team>,
    pub standings: Vec<Division>,
    pub team_ids: Vec<u16>,
    pub date_selector: DateSelector,
    /// Used to skip selecting division names in the table.
    division_row_indices: HashSet<usize>,
}

/// Groups teams into their divisions.
pub struct Division {
    pub name: String,
    pub id: u16,
    pub standings: Vec<Standing>,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
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

/// Standing information per team.
#[derive(Debug, Default)]
pub struct Standing {
    // pub team: Team, // TODO
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
        Self {
            state: TableState::default(),
            standings: Division::create_divisions(),
            team_ids: vec![200, 201, 202, 203, 204, 205],
            date_selector: DateSelector::default(),
            division_row_indices: HashSet::new(),
            favorite_team: None,
        }
    }
}

impl StandingsState {
    /// Update the data from the API.
    pub fn update(&mut self, standings: &StandingsResponse) {
        self.standings = Division::create_table(standings, self.favorite_team);
        self.team_ids = self.generate_ids();

        if self.standings.is_empty() {
            self.state.select(None);
        } else {
            self.reset_selection();
        }
    }

    pub fn reset_selection(&mut self) {
        if let Some(team) = self.favorite_team {
            self.select_favorite_team(team)
        } else if !self.team_ids.is_empty() {
            let offset = 1; // TODO this should be 0 if the standings are pre 1969 since they don't have divisions
            self.state.select(Some(offset));
        }
    }

    /// Set the date from the validated input string from the date picker.
    pub fn set_date_from_valid_input(&mut self, date: NaiveDate) {
        self.date_selector.set_date_from_valid_input(date);
    }

    /// Set the date using Left/Right arrow keys to move a single day at a time.
    pub fn set_date_with_arrows(&mut self, forward: bool) -> NaiveDate {
        self.date_selector.set_date_with_arrows(forward)
    }

    fn generate_ids(&mut self) -> Vec<u16> {
        let mut ids = Vec::with_capacity(36); // 30 teams, 6 divisions
        self.division_row_indices.clear(); // clear previous indices in case they change, e.g. historical standings

        let mut count = 0;
        for division in &self.standings {
            ids.push(division.id);
            self.division_row_indices.insert(count);
            for team in &division.standings {
                ids.push(team.team_id);
            }
            count += 1 + division.standings.len();
        }
        ids
    }

    fn select_favorite_team(&mut self, team: Team) {
        let idx = self
            .standings
            .iter()
            .flat_map(|division| division.standings.iter())
            .enumerate()
            .find(|(_idx, standing)| standing.team_name == team.name)
            .map(|(idx, _standing)| idx + 1);

        self.state.select(idx);
    }

    pub fn get_selected(&self) -> u16 {
        let selected = self.state.selected().unwrap_or(0);
        if let Some(s) = self.team_ids.get(selected) {
            *s
        } else {
            0
        }
    }

    fn skip_division(&self, index: usize) -> bool {
        self.division_row_indices.contains(&index)
    }

    fn move_forward(&self, current: usize) -> usize {
        let len = self.team_ids.len();
        if current >= len - 1 { 0 } else { current + 1 }
    }

    fn move_backward(&self, current: usize) -> usize {
        let len = self.team_ids.len();
        if current == 0 { len - 1 } else { current - 1 }
    }

    pub fn next(&mut self) {
        let len = self.team_ids.len();
        if len == 0 {
            return;
        }

        let start = self.state.selected().unwrap_or(0);
        let mut i = self.move_forward(start);

        if self.skip_division(i) {
            i = self.move_forward(i);
        }

        self.state.select(Some(i));

        // Reset offset when wrapping to beginning
        if i < start {
            self.state = TableState::default();
            self.state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        let len = self.team_ids.len();
        if len == 0 {
            return;
        }

        let start = self.state.selected().unwrap_or(0);
        let mut i = self.move_backward(start);

        if self.skip_division(i) {
            i = self.move_backward(i);
        }

        self.state.select(Some(i));

        // Reset offset when wrapping to end
        if i > start {
            self.state = TableState::default();
            self.state.select(Some(i));
        }
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
    fn create_table(standings: &StandingsResponse, favorite_team: Option<Team>) -> Vec<Division> {
        let mut s: Vec<Division> = standings
            .records
            .iter()
            .map(|r| Division {
                name: DIVISIONS.get(&(r.division.id as u16)).unwrap().to_string(),
                id: r.division.id as u16,
                standings: r
                    .team_records
                    .iter()
                    .map(Standing::from_team_record)
                    .collect(),
            })
            .collect();

        if let Some(team) = favorite_team {
            if let Some(order) = DIVISION_ORDERS.get(&team.division_id) {
                s.sort_by_key(|standing| {
                    order
                        .iter()
                        .position(|&x| x == standing.id)
                        .unwrap_or(usize::MAX)
                });
            }
        } else {
            // ensure display order is the same
            s.sort_by(|a, b| a.id.cmp(&b.id));
        }

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
