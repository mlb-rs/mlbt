use crate::components::standings::Team;
use crate::components::team_page::{RosterRow, TeamGame, TransactionRow};
use crate::state::messages::NetworkRequest;
use crate::state::player_profile::PlayerProfileState;
use chrono::Datelike;
use chrono_tz::Tz;
use mlbt_api::client::StatGroup;
use mlbt_api::schedule::ScheduleResponse;
use mlbt_api::season::GameType;
use mlbt_api::team::{RosterResponse, RosterType, TransactionsResponse};
use std::collections::HashSet;
use tui::widgets::TableState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TeamSection {
    Roster,
    Schedule,
}

pub struct TeamPageState {
    pub team: Team,
    pub date: chrono::NaiveDate,
    pub schedule: Vec<TeamGame>,
    pub schedule_selection: TableState,
    pub today_schedule_idx: usize,
    pub roster: Vec<RosterRow>,
    pub roster_type: RosterType,
    pub transactions: Vec<TransactionRow>,
    pub active_section: TeamSection,
    pub roster_selection: TableState,
    /// includes group header rows
    pub roster_table_len: usize,
    pub roster_header_rows: HashSet<usize>,
    /// table row index -> roster vec index, None for group headers
    pub roster_row_map: Vec<Option<usize>>,
    pub player_profile: Option<PlayerProfileState>,
}

/// Build the mapping from table row indices to roster indices.
/// Returns (total_row_count, header_row_indices, row_map).
fn build_roster_row_map(roster: &[RosterRow]) -> (usize, HashSet<usize>, Vec<Option<usize>>) {
    let mut header_rows = HashSet::new();
    let mut row_map = Vec::new();
    let mut current_group = None;

    for (roster_idx, row) in roster.iter().enumerate() {
        if current_group != Some(row.position_group) {
            current_group = Some(row.position_group);
            header_rows.insert(row_map.len());
            row_map.push(None); // group header
        }
        row_map.push(Some(roster_idx));
    }

    let total = row_map.len();
    (total, header_rows, row_map)
}

impl TeamPageState {
    pub fn from_response(
        team: Team,
        date: chrono::NaiveDate,
        schedule: ScheduleResponse,
        roster: RosterResponse,
        transactions: TransactionsResponse,
        tz: Tz,
    ) -> Self {
        let schedule = TeamGame::from_schedule(schedule, team.id, tz);
        let roster = RosterRow::from_roster(roster);
        let transactions = TransactionRow::from_transactions(transactions);
        let (roster_table_len, roster_header_rows, roster_row_map) = build_roster_row_map(&roster);

        let today_idx = schedule
            .iter()
            .position(|g| !g.is_past)
            .unwrap_or(schedule.len().saturating_sub(1));
        let mut schedule_selection = TableState::default();
        if !schedule.is_empty() {
            schedule_selection.select(Some(today_idx));
            // scroll so ~5 past games are visible above today
            *schedule_selection.offset_mut() = today_idx.saturating_sub(5);
        }

        let mut roster_selection = TableState::default();
        let first = if roster.is_empty() { None } else { Some(1) };
        roster_selection.select(first);

        Self {
            team,
            date,
            schedule,
            schedule_selection,
            today_schedule_idx: today_idx,
            roster,
            roster_type: RosterType::Active,
            transactions,
            active_section: TeamSection::Roster,
            roster_selection,
            roster_table_len,
            roster_header_rows,
            roster_row_map,
            player_profile: None,
        }
    }

    pub fn update_roster(&mut self, roster: RosterResponse, roster_type: RosterType) {
        self.roster = RosterRow::from_roster(roster);
        self.roster_type = roster_type;
        let (len, headers, map) = build_roster_row_map(&self.roster);
        self.roster_table_len = len;
        self.roster_header_rows = headers;
        self.roster_row_map = map;
        let first = if self.roster.is_empty() {
            None
        } else {
            Some(1)
        };
        self.roster_selection.select(first);
    }

    pub fn toggle_section(&mut self) {
        self.active_section = match self.active_section {
            TeamSection::Roster => TeamSection::Schedule,
            TeamSection::Schedule => TeamSection::Roster,
        };
    }

    pub fn next(&mut self) {
        let len = self.active_len();
        if len == 0 {
            return;
        }
        let mut i = (self.selection() + 1) % len;
        if self.active_section == TeamSection::Roster && self.roster_header_rows.contains(&i) {
            i = (i + 1) % len;
        }
        self.set_selection(i);
    }

    pub fn previous(&mut self) {
        let len = self.active_len();
        if len == 0 {
            return;
        }
        let current = self.selection();
        let mut i = if current == 0 { len - 1 } else { current - 1 };
        if self.active_section == TeamSection::Roster && self.roster_header_rows.contains(&i) {
            i = if i == 0 { len - 1 } else { i - 1 };
        }
        self.set_selection(i);
    }

    fn active_len(&self) -> usize {
        match self.active_section {
            TeamSection::Roster => self.roster_table_len,
            TeamSection::Schedule => self.schedule.len(),
        }
    }

    const PAGE_SIZE: usize = 10;

    pub fn page_down(&mut self) {
        for _ in 0..Self::PAGE_SIZE {
            let before = self.selection();
            self.next();
            if self.selection() <= before {
                self.set_selection(before);
                break;
            }
        }
    }

    pub fn page_up(&mut self) {
        for _ in 0..Self::PAGE_SIZE {
            let before = self.selection();
            self.previous();
            if self.selection() >= before {
                self.set_selection(before);
                break;
            }
        }
    }

    fn selection(&self) -> usize {
        match self.active_section {
            TeamSection::Roster => self.roster_selection.selected().unwrap_or(0),
            TeamSection::Schedule => self.schedule_selection.selected().unwrap_or(0),
        }
    }

    fn set_selection(&mut self, idx: usize) {
        match self.active_section {
            TeamSection::Roster => self.roster_selection.select(Some(idx)),
            TeamSection::Schedule => self.schedule_selection.select(Some(idx)),
        }
    }

    /// Get the `RosterRow` for the currently selected table row.
    fn selected_roster_row(&self) -> Option<&RosterRow> {
        let table_idx = self.roster_selection.selected()?;
        let roster_idx = self.roster_row_map.get(table_idx).copied().flatten()?;
        self.roster.get(roster_idx)
    }

    pub fn selected_player_id(&self) -> Option<u64> {
        self.selected_roster_row().map(|r| r.player_id)
    }

    pub fn player_profile_request(&self, date: chrono::NaiveDate) -> Option<NetworkRequest> {
        if self.active_section != TeamSection::Roster {
            return None;
        }
        let row = self.selected_roster_row()?;
        Some(NetworkRequest::PlayerProfile {
            player_id: row.player_id,
            group: row.position_group.stat_group(),
            date,
            game_type: GameType::RegularSeason,
        })
    }

    pub fn roster_toggle_request(&self, roster_type: RosterType) -> Option<NetworkRequest> {
        if self.roster_type == roster_type {
            return None;
        }
        Some(NetworkRequest::TeamRoster {
            team_id: self.team.id,
            season: self.date.year(),
            roster_type,
        })
    }

    pub fn update_player_profile(
        &mut self,
        data: mlbt_api::player::PeopleResponse,
        game_type: GameType,
    ) {
        let group = self
            .selected_roster_row()
            .map(|r| r.position_group.stat_group())
            .unwrap_or(StatGroup::Hitting);
        self.player_profile =
            PlayerProfileState::from_response(data, group, game_type, self.date.year());
    }

    pub fn has_player_profile(&self) -> bool {
        self.player_profile.is_some()
    }

    pub fn close_player_profile(&mut self) {
        self.player_profile = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::team_page::PositionGroup;

    fn make_roster(groups: &[PositionGroup]) -> Vec<RosterRow> {
        groups
            .iter()
            .map(|g| RosterRow {
                player_id: 0,
                number: "1".to_string(),
                name: String::new(),
                position: String::new(),
                position_group: *g,
                bats_throws: String::new(),
                height: String::new(),
                weight: String::new(),
                dob: String::new(),
                status: String::new(),
                status_code: String::new(),
            })
            .collect()
    }

    fn nav_state(roster_groups: &[PositionGroup], schedule_len: usize) -> TeamPageState {
        let roster = make_roster(roster_groups);
        let (len, headers, map) = build_roster_row_map(&roster);
        let mut roster_selection = TableState::default();
        if !roster.is_empty() {
            roster_selection.select(Some(1));
        }
        let mut schedule_selection = TableState::default();
        if schedule_len > 0 {
            schedule_selection.select(Some(0));
        }
        TeamPageState {
            team: Team::default(),
            date: chrono::NaiveDate::default(),
            schedule: vec![
                TeamGame {
                    date: chrono::NaiveDate::default(),
                    date_display: String::new(),
                    opponent: String::new(),
                    time_or_score: String::new(),
                    is_home: false,
                    is_past: false,
                };
                schedule_len
            ],
            schedule_selection,
            today_schedule_idx: 0,
            roster,
            roster_type: RosterType::Active,
            transactions: vec![],
            active_section: TeamSection::Roster,
            roster_selection,
            roster_table_len: len,
            roster_header_rows: headers,
            roster_row_map: map,
            player_profile: None,
        }
    }

    #[test]
    fn roster_next_skips_headers() {
        // table: [header(0), A(1), header(2), B(3)]
        let mut s = nav_state(&[PositionGroup::Pitcher, PositionGroup::Catcher], 0);
        assert_eq!(s.selection(), 1);
        s.next();
        assert_eq!(s.selection(), 3); // skipped header at 2
        s.next();
        assert_eq!(s.selection(), 1); // wrapped, skipped header at 0
    }

    #[test]
    fn schedule_does_not_skip() {
        let mut s = nav_state(&[], 5);
        s.active_section = TeamSection::Schedule;
        s.next();
        assert_eq!(s.selection(), 1);
        s.next();
        assert_eq!(s.selection(), 2);
    }

    #[test]
    fn page_down_does_not_wrap() {
        // table: [header(0), X(1), X(2), X(3)]
        let mut s = nav_state(&[PositionGroup::Pitcher; 3], 0);
        s.page_down();
        assert_eq!(s.selection(), 3);
    }

    #[test]
    fn page_up_does_not_wrap() {
        let mut s = nav_state(&[PositionGroup::Pitcher; 3], 0);
        s.set_selection(3);
        s.page_up();
        assert_eq!(s.selection(), 1);
    }
}
