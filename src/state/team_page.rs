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
use std::sync::Arc;
use tui::widgets::TableState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TeamSection {
    Roster,
    Schedule,
    Transactions,
}

pub struct TeamPageState {
    pub team: Team,
    pub date: chrono::NaiveDate,
    pub schedule: Vec<TeamGame>,
    pub schedule_selection: TableState,
    pub roster: Vec<RosterRow>,
    pub roster_type: RosterType,
    pub transactions: Vec<TransactionRow>,
    pub selected_transaction: usize,
    pub transaction_scroll: u16,
    pub active_section: TeamSection,
    pub roster_selection: TableState,
    /// includes group header rows
    pub roster_table_len: usize,
    pub roster_header_rows: HashSet<usize>,
    /// table row index -> roster vec index, None for group headers
    pub roster_row_map: Vec<Option<usize>>,
    pub player_profile: Option<PlayerProfileState>,
    pub show_calendar: bool,
}

impl TeamPageState {
    const PAGE_SIZE: usize = 10;
    pub const TRANSACTION_DATE_WIDTH: usize = 8;

    pub fn from_response(
        team: Team,
        date: chrono::NaiveDate,
        schedule: &ScheduleResponse,
        roster: &RosterResponse,
        transactions: &TransactionsResponse,
        tz: Tz,
    ) -> Self {
        let schedule = TeamGame::from_schedule(schedule, team.id, date, tz);
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
            roster,
            roster_type: RosterType::Active,
            transactions,
            selected_transaction: 0,
            transaction_scroll: 0,
            active_section: TeamSection::Roster,
            roster_selection,
            roster_table_len,
            roster_header_rows,
            roster_row_map,
            player_profile: None,
            show_calendar: true,
        }
    }

    pub fn update_roster(&mut self, roster: &RosterResponse, roster_type: RosterType) {
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

    pub fn refresh_schedule_times(&mut self, tz: Tz) {
        for game in &mut self.schedule {
            game.refresh_time_or_score(tz);
        }
    }

    pub fn next_section(&mut self) {
        self.active_section = match self.active_section {
            TeamSection::Roster => TeamSection::Schedule,
            TeamSection::Schedule => TeamSection::Transactions,
            TeamSection::Transactions => TeamSection::Roster,
        };
    }

    pub fn previous_section(&mut self) {
        self.active_section = match self.active_section {
            TeamSection::Roster => TeamSection::Transactions,
            TeamSection::Schedule => TeamSection::Roster,
            TeamSection::Transactions => TeamSection::Schedule,
        };
    }

    pub fn toggle_calendar(&mut self) {
        self.show_calendar = !self.show_calendar;
    }

    pub fn next(&mut self) {
        let len = self.active_len();
        if len == 0 {
            return;
        }
        let start = self.selection();
        let mut i = if start >= len - 1 { 0 } else { start + 1 };
        if self.active_section == TeamSection::Roster && self.roster_header_rows.contains(&i) {
            i = if i >= len - 1 { 0 } else { i + 1 };
        }
        self.set_selection(i);
        if i < start {
            self.reset_active_table_state(i);
        }
    }

    pub fn previous(&mut self) {
        let len = self.active_len();
        if len == 0 {
            return;
        }
        let start = self.selection();
        let mut i = if start == 0 { len - 1 } else { start - 1 };
        if self.active_section == TeamSection::Roster && self.roster_header_rows.contains(&i) {
            i = if i == 0 { len - 1 } else { i - 1 };
        }
        self.set_selection(i);
        if i > start {
            self.reset_active_table_state(i);
        }
    }

    fn active_len(&self) -> usize {
        match self.active_section {
            TeamSection::Roster => self.roster_table_len,
            TeamSection::Schedule => self.schedule.len(),
            TeamSection::Transactions => self.transactions.len(),
        }
    }

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
            TeamSection::Transactions => self.selected_transaction,
        }
    }

    fn set_selection(&mut self, idx: usize) {
        match self.active_section {
            TeamSection::Roster => self.roster_selection.select(Some(idx)),
            TeamSection::Schedule => self.schedule_selection.select(Some(idx)),
            TeamSection::Transactions => self.selected_transaction = idx,
        }
    }

    /// Reset the active table state and re-select, clearing the scroll offset.
    fn reset_active_table_state(&mut self, idx: usize) {
        match self.active_section {
            TeamSection::Roster => {
                self.roster_selection = TableState::default();
                self.roster_selection.select(Some(idx));
            }
            TeamSection::Schedule => {
                self.schedule_selection = TableState::default();
                self.schedule_selection.select(Some(idx));
            }
            TeamSection::Transactions => {
                self.selected_transaction = idx;
                self.transaction_scroll = 0;
            }
        }
    }

    /// Get the `RosterRow` for the currently selected table row.
    fn selected_roster_row(&self) -> Option<&RosterRow> {
        let table_idx = self.roster_selection.selected()?;
        let roster_idx = self.roster_row_map.get(table_idx).copied().flatten()?;
        self.roster.get(roster_idx)
    }

    pub fn player_profile_request(&self) -> Option<NetworkRequest> {
        if self.active_section != TeamSection::Roster {
            return None;
        }
        let row = self.selected_roster_row()?;
        Some(NetworkRequest::PlayerProfile {
            player_id: row.player_id,
            group: row.position_group.stat_group(),
            date: self.date,
            game_type: GameType::RegularSeason,
        })
    }

    pub fn roster_toggle_request(&self) -> NetworkRequest {
        let roster_type = match self.roster_type {
            RosterType::Active => RosterType::FortyMan,
            RosterType::FortyMan => RosterType::Active,
        };
        NetworkRequest::TeamRoster {
            team_id: self.team.id,
            season: self.date.year(),
            roster_type,
        }
    }

    pub fn update_player_profile(
        &mut self,
        data: Arc<mlbt_api::player::PeopleResponse>,
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

    /// Update the transaction section scroll offset to keep the selected transaction visible.
    pub fn update_transaction_scroll(&mut self, area_width: u16, area_height: u16) {
        let selected = self.selected_transaction;
        let widths = self.transaction_line_widths();
        if selected >= widths.len() {
            return;
        }

        // estimate how many visual lines each transaction occupies after wrap, then find the visual
        // line where the selected transaction starts
        let width = area_width.max(1) as usize;
        let wrapped_height = |line_width: usize| -> usize { line_width.div_ceil(width).max(1) };
        let sel_start: usize = widths[..selected].iter().map(|&w| wrapped_height(w)).sum();
        let sel_height = wrapped_height(widths[selected]);

        // adjust scroll just enough to keep the selected transaction in view
        let scroll = self.transaction_scroll as usize;
        let visible = area_height as usize;
        self.transaction_scroll = if sel_start < scroll {
            // scrolled past it — snap up
            sel_start
        } else if sel_start + sel_height > scroll + visible {
            // below viewport — snap down
            (sel_start + sel_height).saturating_sub(visible)
        } else {
            // already visible — no change
            scroll
        } as u16;
    }

    fn transaction_line_widths(&self) -> Vec<usize> {
        self.transactions
            .iter()
            .map(|t| Self::TRANSACTION_DATE_WIDTH + t.description.len())
            .collect()
    }
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
                    start_time_utc: None,
                    is_home: false,
                    is_past: false,
                    is_win: None,
                };
                schedule_len
            ],
            schedule_selection,
            roster,
            roster_type: RosterType::Active,
            transactions: vec![],
            selected_transaction: 0,
            transaction_scroll: 0,
            active_section: TeamSection::Roster,
            roster_selection,
            roster_table_len: len,
            roster_header_rows: headers,
            roster_row_map: map,
            player_profile: None,
            show_calendar: true,
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

    #[test]
    fn next_wrap_resets_offset() {
        // table: [header(0), A(1), header(2), B(3)]
        let mut s = nav_state(&[PositionGroup::Pitcher, PositionGroup::Catcher], 0);
        // simulate having scrolled down
        *s.roster_selection.offset_mut() = 2;
        s.set_selection(3);

        // wrap from last row back to first
        s.next();
        assert_eq!(s.selection(), 1);
        assert_eq!(*s.roster_selection.offset_mut(), 0);
    }

    #[test]
    fn previous_wrap_resets_offset() {
        // table: [header(0), A(1), header(2), B(3)]
        let mut s = nav_state(&[PositionGroup::Pitcher, PositionGroup::Catcher], 0);
        assert_eq!(s.selection(), 1);

        // wrap from first row to last
        s.previous();
        assert_eq!(s.selection(), 3);
        assert_eq!(*s.roster_selection.offset_mut(), 0);
    }

    /// Build a transaction state with descriptions of the given lengths.
    /// Line width = TRANSACTION_DATE_WIDTH + description length.
    fn make_transaction_state(desc_lens: &[usize]) -> TeamPageState {
        let mut s = nav_state(&[], 0);
        s.active_section = TeamSection::Transactions;
        s.transactions = desc_lens
            .iter()
            .map(|&len| TransactionRow {
                date: String::new(),
                description: "x".repeat(len),
            })
            .collect();
        s
    }

    #[test]
    fn transaction_scroll_down() {
        // 5 lines, each 8+12=20 wide, area 40 wide x 3 tall — no wrapping
        let mut s = make_transaction_state(&[12; 5]);
        s.selected_transaction = 4;
        s.update_transaction_scroll(40, 3);
        assert_eq!(s.transaction_scroll, 2);
    }

    #[test]
    fn transaction_scroll_up() {
        let mut s = make_transaction_state(&[12; 5]);
        s.selected_transaction = 1;
        s.transaction_scroll = 3;
        s.update_transaction_scroll(40, 3);
        assert_eq!(s.transaction_scroll, 1);
    }

    #[test]
    fn transaction_scroll_no_change_when_visible() {
        let mut s = make_transaction_state(&[12; 5]);
        s.selected_transaction = 2;
        s.transaction_scroll = 1;
        s.update_transaction_scroll(40, 3);
        // sel_start=2, scroll=1, visible=[1,2,3] — already visible
        assert_eq!(s.transaction_scroll, 1);
    }

    #[test]
    fn transaction_scroll_with_wrapping_lines() {
        // line 0: 8+72=80 wide in 40-col area -> 2 visual lines
        // line 1: 8+12=20 -> 1 visual line
        // line 2: 8+12=20 -> 1 visual line
        let mut s = make_transaction_state(&[72, 12, 12]);
        s.selected_transaction = 2;
        s.update_transaction_scroll(40, 3);
        // sel_start = 2+1 = 3, sel_height = 1, visible = 3
        assert_eq!(s.transaction_scroll, 1);
    }

    #[test]
    fn section_cycles() {
        let mut s = nav_state(&[], 0);
        assert_eq!(s.active_section, TeamSection::Roster);
        s.next_section();
        assert_eq!(s.active_section, TeamSection::Schedule);
        s.next_section();
        assert_eq!(s.active_section, TeamSection::Transactions);
        s.next_section();
        assert_eq!(s.active_section, TeamSection::Roster);
        s.previous_section();
        assert_eq!(s.active_section, TeamSection::Transactions);
        s.previous_section();
        assert_eq!(s.active_section, TeamSection::Schedule);
    }

    #[test]
    fn refresh_schedule_times_updates_only_upcoming_games() {
        let mut s = nav_state(&[], 0);
        s.schedule = vec![
            TeamGame {
                date: chrono::NaiveDate::default(),
                date_display: String::new(),
                opponent: String::new(),
                time_or_score: "7:10 PM".to_string(),
                start_time_utc: Some(
                    chrono::NaiveDate::from_ymd_opt(2025, 3, 28)
                        .unwrap()
                        .and_hms_opt(23, 10, 0)
                        .unwrap()
                        .and_utc(),
                ),
                is_home: false,
                is_past: false,
                is_win: None,
            },
            TeamGame {
                date: chrono::NaiveDate::default(),
                date_display: String::new(),
                opponent: String::new(),
                time_or_score: "4-3 W".to_string(),
                start_time_utc: None,
                is_home: false,
                is_past: true,
                is_win: Some(true),
            },
        ];

        s.refresh_schedule_times(chrono_tz::US::Pacific);

        assert_eq!(s.schedule[0].time_or_score, "4:10 pm");
        assert_eq!(s.schedule[1].time_or_score, "4-3 W");
    }
}
