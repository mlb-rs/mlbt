use crate::components::constants::{DIVISION_ORDERS, DIVISIONS, TEAM_IDS};
use crate::components::date_selector::DateSelector;
use chrono::NaiveDate;
use mlb_api::standings::{RecordElement, StandingsResponse, TeamRecord};
use std::collections::HashSet;
use std::string::ToString;
use tui::prelude::{Color, Stylize};
use tui::widgets::{Cell, TableState};

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

/// Standing information per team.
#[derive(Debug, Default)]
pub struct Standing {
    pub team: Team,
    pub wins: u8,
    pub losses: u8,
    pub winning_percentage: String,
    pub games_back: String,
    pub wild_card_games_back: String,
    pub last_10: String,
    pub streak: String,
    pub runs_scored: u16,
    pub runs_allowed: u16,
    pub run_differential: i16,
    pub xwl: String,
    pub home: String,
    pub away: String,
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
                ids.push(team.team.id);
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
            .find(|(_idx, standing)| standing.team.id == team.id)
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
                name: DIVISIONS[&id].to_string(),
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
            team: TEAM_IDS
                .get(&team.team.name.as_str())
                .cloned()
                .unwrap_or_default(),
            wins: team.wins,
            losses: team.losses,
            winning_percentage: team.winning_percentage.clone(),
            games_back: team.games_back.clone(),
            wild_card_games_back: team.wild_card_games_back.clone(),
            last_10,
            streak,
            runs_scored: team.runs_scored,
            runs_allowed: team.runs_allowed,
            run_differential: team.run_differential,
            xwl,
            home,
            away,
        }
    }

    pub fn to_cells(&self) -> Vec<Cell> {
        let (prefix, rdiff_color) = match self.run_differential.signum() {
            1 => ("+", Color::Green),
            -1 => ("", Color::Red),
            _ => ("", Color::White),
        };
        vec![
            self.team.name.to_string().into(),
            self.wins.to_string().into(),
            self.losses.to_string().into(),
            self.winning_percentage.clone().into(),
            self.games_back.clone().into(),
            self.wild_card_games_back.clone().into(),
            self.last_10.clone().into(),
            self.streak.clone().into(),
            self.runs_scored.to_string().into(),
            self.runs_allowed.to_string().into(),
            Cell::from(format!("{}{}", prefix, self.run_differential)).fg(rdiff_color),
            self.xwl.clone().into(),
            self.home.clone().into(),
            self.away.clone().into(),
        ]
    }
}
