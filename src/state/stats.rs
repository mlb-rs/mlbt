use crate::components::constants::lookup_team_by_id;
use crate::components::date_selector::DateSelector;
use crate::components::stats::search::SearchState;
use crate::components::stats::table::{
    PLAYER_COLUMN_NAME, Sort, StatType, StatsTable, TEAM_COLUMN_NAME, TableData, TeamOrPlayer,
};
use crate::state::messages::NetworkRequest;
use crate::state::player_profile::PlayerProfileState;
use crate::state::team_page::TeamPageState;
use chrono::{Datelike, NaiveDate};
use chrono_tz::Tz;
use mlbt_api::client::StatGroup;
use mlbt_api::player::PeopleResponse;
use mlbt_api::schedule::ScheduleResponse;
use mlbt_api::season::GameType;
use mlbt_api::stats::StatsResponse;
use mlbt_api::team::{RosterResponse, RosterType, TransactionsResponse};
use std::collections::HashMap;
use std::sync::Arc;
use tui::widgets::TableState;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ActivePane {
    #[default]
    Data,
    Options,
}

/// Identifies a stat table view. Qualification is excluded since it only filters rows.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ViewKey {
    group: StatGroup,
    team_player: TeamOrPlayer,
}

/// Sort and column visibility for one `ViewKey`.
#[derive(Clone, Default)]
struct ViewPrefs {
    /// Sort column + order. `None` falls back to the default in `Sort::new`.
    sort: Option<Sort>,
    /// Column name to active flag, only for columns that have been toggled.
    column_overrides: HashMap<String, bool>,
}

/// Stores the state for rendering the stats.
pub struct StatsState {
    /// TableState for the options sidebar column selection.
    pub options_state: TableState,
    /// TableState for the main data table row selection.
    pub data_state: TableState,
    /// Which pane is currently focused.
    pub active_pane: ActivePane,
    /// Pane that was active before search opened, used to restore on cancel.
    pub search_previous_pane: Option<ActivePane>,
    /// The stat type combination of team/player, pitching/hitting, and all/qualified.
    pub stat_type: StatType,
    /// The stats table data and cache.
    pub table: StatsTable,
    /// Whether to display the side bar for toggling stats
    pub show_options: bool,
    /// Date selector for viewing stats on a specific date.
    pub date_selector: DateSelector,
    /// Visible row count in the data table, updated during render.
    pub visible_rows: usize,
    /// Search state for players or teams.
    pub search: SearchState,
    /// Active player profile view. When Some, renders full-page replacing the stats table.
    pub player_profile: Option<PlayerProfileState>,
    /// Active team page view. When Some, renders full-page replacing the stats table.
    pub team_page: Option<TeamPageState>,
    /// Sort and column visibility per `ViewKey`, restored on `update()`.
    view_prefs: HashMap<ViewKey, ViewPrefs>,
    /// Last `ViewKey` seen by `update()`. Used to keep the options pane row when only qualification
    /// toggles, since the stats columns don't change.
    last_view_key: Option<ViewKey>,
}

impl Default for StatsState {
    fn default() -> Self {
        let stat_type = StatType::default();
        let mut ss = StatsState {
            options_state: TableState::default(),
            data_state: TableState::default(),
            active_pane: ActivePane::default(),
            search_previous_pane: None,
            stat_type,
            table: StatsTable::new(stat_type),
            show_options: true,
            date_selector: DateSelector::default(),
            visible_rows: 0,
            search: SearchState::default(),
            player_profile: None,
            team_page: None,
            view_prefs: HashMap::new(),
            last_view_key: None,
        };
        ss.options_state.select(Some(0));
        ss.data_state.select(Some(0));
        ss
    }
}

impl StatsState {
    pub fn update(&mut self, stats: &StatsResponse) {
        let current_key = self.view_key();
        let same_view = self.last_view_key == Some(current_key);
        self.last_view_key = Some(current_key);

        self.player_profile = None;
        self.table.load(stats, self.stat_type);
        self.apply_view_prefs();
        self.data_state.select(Some(0));
        // Only reset the options pane row selection if the columns could differ.
        if !same_view {
            self.options_state.select(Some(0));
        }
        // Clear search state since the underlying data has changed.
        self.search.close();
        self.search_previous_pane = None;
    }

    fn view_key(&self) -> ViewKey {
        ViewKey {
            group: self.stat_type.group,
            team_player: self.stat_type.team_player,
        }
    }

    /// Restore saved sort and column visibility for the current view. Called after `table.load()`
    /// has reset everything to defaults.
    fn apply_view_prefs(&mut self) {
        let Some(prefs) = self.view_prefs.get(&self.view_key()) else {
            return;
        };
        for (name, active) in &prefs.column_overrides {
            if let Some(entry) = self.table.columns.get_mut(name) {
                entry.active = *active;
            }
        }

        // Only restore the saved sort if its column still exists and is visible.
        if let Some(sort) = &prefs.sort
            && let Some(name) = &sort.column_name
            && self
                .table
                .columns
                .get(name)
                .is_some_and(|entry| entry.active)
        {
            self.table.sorting = sort.clone();
        }
    }

    pub fn has_player_profile(&self) -> bool {
        self.player_profile.is_some()
    }

    pub fn has_team_page(&self) -> bool {
        self.team_page.is_some()
    }

    /// Close the top layer overlay (player profile or team page).
    pub fn close_overlay(&mut self) {
        if let Some(tp) = &mut self.team_page {
            if tp.player_profile.is_some() {
                tp.player_profile = None;
            } else {
                self.team_page = None;
            }
        } else {
            self.player_profile = None;
        }
    }

    pub fn update_team_page(
        &mut self,
        team_id: u16,
        date: NaiveDate,
        schedule: &ScheduleResponse,
        roster: &RosterResponse,
        transactions: &TransactionsResponse,
        tz: Tz,
    ) {
        let team = lookup_team_by_id(team_id).unwrap_or_default();
        self.team_page = Some(TeamPageState::from_response(
            team,
            date,
            schedule,
            roster,
            transactions,
            tz,
        ));
    }

    pub fn update_team_roster(
        &mut self,
        team_id: u16,
        roster: &RosterResponse,
        roster_type: RosterType,
    ) {
        if let Some(tp) = &mut self.team_page
            && tp.team.id == team_id
        {
            tp.update_roster(roster, roster_type);
        }
    }

    pub fn update_team_player_profile(&mut self, data: Arc<PeopleResponse>, game_type: GameType) {
        if let Some(tp) = &mut self.team_page {
            tp.update_player_profile(data, game_type);
        }
    }

    pub fn update_player_profile(&mut self, data: Arc<PeopleResponse>, game_type: GameType) {
        let season_year = self.date_selector.date.year();
        self.player_profile =
            PlayerProfileState::from_response(data, self.stat_type.group, game_type, season_year);
    }

    /// Returns the request to open the selected row (player profile or team page).
    pub fn open_selected_request(&self) -> Option<NetworkRequest> {
        if self.stat_type.team_player == TeamOrPlayer::Team {
            let team_id = self.get_selected_id()? as u16;
            return Some(NetworkRequest::TeamPage {
                team_id,
                date: self.date_selector.date,
            });
        }
        let player_id = self.get_selected_id()?;
        Some(NetworkRequest::PlayerProfile {
            player_id,
            group: self.stat_type.group,
            date: self.date_selector.date,
            game_type: GameType::RegularSeason,
        })
    }

    /// Set the date from the validated input string from the date picker.
    pub fn set_date_from_valid_input(&mut self, date: NaiveDate) {
        self.date_selector.set_date_from_valid_input(date);
        self.select(Some(0));
    }

    /// Set the date using Left/Right arrow keys to move a single day at a time.
    pub fn set_date_with_arrows(&mut self, forward: bool) -> NaiveDate {
        self.date_selector.set_date_with_arrows(forward)
    }

    /// Run fuzzy matching against the name column and update search.matched_indices.
    pub fn update_search_matches(&mut self) {
        self.table.invalidate_cache();
        let name_key = match self.stat_type.team_player {
            TeamOrPlayer::Team => TEAM_COLUMN_NAME,
            TeamOrPlayer::Player => PLAYER_COLUMN_NAME,
        };
        let empty = Vec::new();
        let names = self
            .table
            .columns
            .get(name_key)
            .map(|entry| &entry.rows)
            .unwrap_or(&empty);
        self.search.update_matches(names);
    }

    /// Open search and force navigation to the data pane while searching.
    pub fn open_search(&mut self) {
        if !self.search.is_open {
            self.search_previous_pane = Some(self.active_pane);
        }
        self.active_pane = ActivePane::Data;
        self.data_state.select(Some(0));
        self.search.open();
    }

    /// Submit search results and keep focus on the data pane.
    pub fn submit_search(&mut self) {
        self.search.submit();
        self.search_previous_pane = None;
        self.active_pane = ActivePane::Data;
    }

    /// Cancel search and restore the pane that was active before opening search.
    pub fn cancel_search(&mut self) {
        self.search.close();
        self.table.invalidate_cache();
        if let Some(previous) = self.search_previous_pane.take() {
            self.active_pane = if previous == ActivePane::Options && !self.show_options {
                ActivePane::Data
            } else {
                previous
            };
        }
    }

    /// Generate the table data, using the cache if available.
    pub fn generate_table(&mut self) -> Arc<TableData> {
        let filter = if self.search.is_filtering() {
            Some(self.search.matched_indices.as_slice())
        } else {
            None
        };
        self.table.generate(filter)
    }

    pub fn toggle_options(&mut self) {
        self.show_options = !self.show_options;
        if !self.show_options {
            self.active_pane = ActivePane::Data;
        }
    }

    pub fn switch_pane(&mut self) {
        if !self.show_options {
            return;
        }
        self.active_pane = match self.active_pane {
            ActivePane::Data => ActivePane::Options,
            ActivePane::Options => ActivePane::Data,
        };
    }

    /// Toggle the visibility of the stat column that is selected.
    pub fn toggle_stat(&mut self) {
        let idx = self.options_state.selected().unwrap_or_default();
        self.table.toggle_stat(idx);
        let key = self.view_key();
        let prefs = self.view_prefs.entry(key).or_default();
        if let Some((name, entry)) = self.table.columns.get_index(idx) {
            prefs.column_overrides.insert(name.clone(), entry.active);
        }
        // StatsTable.toggle_stat clears the sort if the sort column was toggled off. Mirror that so
        // it doesn't come back on the next view switch.
        if self.table.sorting.column_name.is_none() {
            prefs.sort = None;
        }
    }

    /// Sort the table by the selected stat.
    pub fn store_sort_column(&mut self) {
        let Some(idx) = self.options_state.selected() else {
            return;
        };
        self.table.store_sort_column(idx);
        let key = self.view_key();
        let prefs = self.view_prefs.entry(key).or_default();
        prefs.sort = if self.table.sorting.column_name.is_some() {
            Some(self.table.sorting.clone())
        } else {
            None
        };
    }

    /// Get the player or team id for the currently selected row.
    pub fn get_selected_id(&self) -> Option<u64> {
        let selected = self.data_state.selected()?;
        let (_, ids, _) = self.table.cached()?.as_ref();
        ids.get(selected).copied()
    }

    /// Returns the total number of data rows, ignoring any search filter.
    pub fn total_row_count(&self) -> usize {
        self.table.total_row_count()
    }

    /// Returns the number of visible data rows, accounting for search filtering.
    fn row_count(&self) -> usize {
        if self.search.is_filtering() {
            return self.search.matched_indices.len();
        }
        self.total_row_count()
    }

    /// Reset data selection to the first visible row for the current view.
    pub fn reset_data_selection(&mut self) {
        if self.row_count() > 0 {
            self.data_state.select(Some(0));
        } else {
            self.data_state.select(None);
        }
    }

    pub fn next(&mut self) {
        let len = self.active_pane_len();
        if len == 0 {
            return;
        }

        let next = match self.selected() {
            Some(i) if i >= len - 1 => 0,
            Some(i) => i + 1,
            None => 0,
        };

        self.select(Some(next));
    }

    pub fn previous(&mut self) {
        let len = self.active_pane_len();
        if len == 0 {
            return;
        }

        let previous = match self.selected() {
            Some(0) => len - 1,
            Some(i) => i - 1,
            None => 0,
        };

        self.select(Some(previous));
    }

    fn select(&mut self, index: Option<usize>) {
        match self.active_pane {
            ActivePane::Data => self.data_state.select(index),
            ActivePane::Options => self.options_state.select(index),
        }
    }

    fn selected(&self) -> Option<usize> {
        match self.active_pane {
            ActivePane::Data => self.data_state.selected(),
            ActivePane::Options => self.options_state.selected(),
        }
    }

    fn active_pane_len(&self) -> usize {
        match self.active_pane {
            ActivePane::Data => self.row_count(),
            ActivePane::Options => self.table.columns.len(),
        }
    }

    pub fn page_down(&mut self) {
        if !self.can_page_data() {
            return;
        }
        // The last visible row becomes the first visible row
        let len = self.row_count();
        let offset = self.data_state.offset();
        let last_visible = (offset + self.visible_rows - 1).min(len - 1);
        self.select_data_row(last_visible);
    }

    pub fn page_up(&mut self) {
        if !self.can_page_data() {
            return;
        }
        // The first visible row becomes the last visible row
        let offset = self.data_state.offset();
        let new_offset = offset.saturating_sub(self.visible_rows - 1);
        self.select_data_row(new_offset);
    }

    fn can_page_data(&self) -> bool {
        self.active_pane == ActivePane::Data && self.row_count() > 0 && self.visible_rows > 0
    }

    fn select_data_row(&mut self, index: usize) {
        *self.data_state.offset_mut() = index;
        self.data_state.select(Some(index));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::stats::table::TableEntry;

    fn state_with(n_rows: usize, n_cols: usize) -> StatsState {
        let mut s = StatsState::default();
        for i in 0..n_cols {
            s.table.columns.insert(
                format!("c{i}"),
                TableEntry {
                    description: String::new(),
                    active: true,
                    rows: (0..n_rows).map(|r| r.to_string()).collect(),
                },
            );
        }
        s
    }

    #[test]
    fn next_wraps_at_end() {
        let mut s = state_with(3, 1);
        s.data_state.select(Some(2));
        s.next();
        assert_eq!(s.data_state.selected(), Some(0));
    }

    #[test]
    fn previous_wraps_at_start() {
        let mut s = state_with(3, 1);
        s.data_state.select(Some(0));
        s.previous();
        assert_eq!(s.data_state.selected(), Some(2));
    }

    #[test]
    fn next_targets_active_pane() {
        let mut s = state_with(5, 3);
        s.active_pane = ActivePane::Options;
        s.options_state.select(Some(0));
        s.next();
        assert_eq!(s.options_state.selected(), Some(1));
        assert_eq!(s.data_state.selected(), Some(0));
    }

    #[test]
    fn page_down_jumps_to_last_visible() {
        let mut s = state_with(20, 1);
        s.visible_rows = 5;
        s.data_state.select(Some(0));
        s.page_down();
        assert_eq!(s.data_state.selected(), Some(4));
    }

    #[test]
    fn page_up_reverses_page_down() {
        let mut s = state_with(20, 1);
        s.visible_rows = 5;
        *s.data_state.offset_mut() = 10;
        s.data_state.select(Some(10));
        s.page_up();
        assert_eq!(s.data_state.selected(), Some(6));
    }

    #[test]
    fn page_down_clamps_at_end() {
        let mut s = state_with(20, 1);
        s.visible_rows = 5;
        *s.data_state.offset_mut() = 19;
        s.data_state.select(Some(19));
        s.page_down();
        assert_eq!(s.data_state.selected(), Some(19));
    }

    #[test]
    fn page_up_clamps_at_start() {
        let mut s = state_with(20, 1);
        s.visible_rows = 5;
        s.data_state.select(Some(0));
        s.page_up();
        assert_eq!(s.data_state.selected(), Some(0));
    }

    #[test]
    fn page_noop_on_options_pane() {
        let mut s = state_with(20, 1);
        s.visible_rows = 5;
        s.active_pane = ActivePane::Options;
        s.data_state.select(Some(0));
        s.page_down();
        assert_eq!(s.data_state.selected(), Some(0));
    }
}
