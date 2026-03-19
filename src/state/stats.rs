use crate::components::date_selector::DateSelector;
use crate::components::stats::search::SearchState;
use crate::components::stats::table::{
    PLAYER_COLUMN_NAME, StatType, StatsTable, TEAM_COLUMN_NAME, TableData, TeamOrPlayer,
};
use crate::state::messages::NetworkRequest;
use crate::state::player_profile::PlayerProfileState;
use chrono::{Datelike, NaiveDate};
use mlbt_api::client::StatGroup;
use mlbt_api::player::PeopleResponse;
use mlbt_api::season::GameType;
use mlbt_api::stats::StatsResponse;
use std::sync::Arc;
use tui::widgets::TableState;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ActivePane {
    Data,
    #[default]
    Options,
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
    /// The stat type combination of team/player and pitching/hitting.
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
}

impl Default for StatsState {
    fn default() -> Self {
        let mut ss = StatsState {
            options_state: TableState::default(),
            data_state: TableState::default(),
            active_pane: ActivePane::default(),
            search_previous_pane: None,
            stat_type: StatType {
                group: StatGroup::Hitting,
                team_player: TeamOrPlayer::Player,
            },
            table: StatsTable::default(),
            show_options: true,
            date_selector: DateSelector::default(),
            visible_rows: 0,
            search: SearchState::default(),
            player_profile: None,
        };
        ss.options_state.select(Some(0));
        ss
    }
}

impl StatsState {
    pub fn update(&mut self, stats: &StatsResponse) {
        self.player_profile = None;
        self.table.load(stats, self.stat_type.team_player);
        self.data_state.select(Some(0));
        // Clear search state since the underlying data has changed.
        self.search.close();
        self.search_previous_pane = None;
    }

    pub fn has_player_profile(&self) -> bool {
        self.player_profile.is_some()
    }

    pub fn close_player_profile(&mut self) {
        self.player_profile = None;
    }

    pub fn update_player_profile(&mut self, data: PeopleResponse, game_type: GameType) {
        let season_year = self.date_selector.date.year();
        self.player_profile =
            PlayerProfileState::from_response(data, self.stat_type.group, game_type, season_year);
    }

    /// Returns the info needed to load a player profile for the currently selected row.
    /// Returns None if in team mode or no row is selected.
    pub fn player_profile_request(&self) -> Option<NetworkRequest> {
        // TODO support team profiles
        if self.stat_type.team_player != TeamOrPlayer::Player {
            return None;
        }
        let player_id = self.get_selected_id()?;
        Some(NetworkRequest::PlayerProfile {
            player_id,
            group: self.stat_type.group,
            date: self.date_selector.date,
            game_type: GameType::RegularSeason,
        })
    }

    /// Returns a request to reload the current profile with a toggled game type.
    pub fn toggle_profile_game_type(&mut self) -> Option<NetworkRequest> {
        let profile = self.player_profile.as_mut()?;
        profile.toggle_game_type();
        Some(NetworkRequest::PlayerProfile {
            player_id: profile.profile.id,
            group: profile.stat_group,
            date: self.date_selector.date,
            game_type: profile.game_type,
        })
    }

    /// Set the date from the validated input string from the date picker.
    pub fn set_date_from_valid_input(&mut self, date: NaiveDate) {
        self.date_selector.set_date_from_valid_input(date);
        self.select(Some(0));
    }

    fn select(&mut self, index: Option<usize>) {
        match self.active_pane {
            ActivePane::Data => self.data_state.select(index),
            ActivePane::Options => self.options_state.select(index),
        }
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
    }

    /// Sort the table by the selected stat.
    pub fn store_sort_column(&mut self) {
        let Some(idx) = self.options_state.selected() else {
            return;
        };
        self.table.store_sort_column(idx);
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
        let len = match self.active_pane {
            ActivePane::Data => self.row_count(),
            ActivePane::Options => self.table.columns.len(),
        };
        if len == 0 {
            return;
        }
        let selected = match self.active_pane {
            ActivePane::Data => self.data_state.selected(),
            ActivePane::Options => self.options_state.selected(),
        };
        let i = match selected {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.select(Some(i));
    }

    pub fn page_down(&mut self) {
        if self.active_pane != ActivePane::Data {
            return;
        }
        let len = self.row_count();
        if len == 0 || self.visible_rows == 0 {
            return;
        }
        // The last visible row becomes the first visible row
        let offset = self.data_state.offset();
        let last_visible = (offset + self.visible_rows - 1).min(len - 1);
        *self.data_state.offset_mut() = last_visible;
        self.data_state.select(Some(last_visible));
    }

    pub fn page_up(&mut self) {
        if self.active_pane != ActivePane::Data {
            return;
        }
        let len = self.row_count();
        if len == 0 || self.visible_rows == 0 {
            return;
        }
        // The first visible row becomes the last visible row
        let offset = self.data_state.offset();
        let new_offset = offset.saturating_sub(self.visible_rows - 1);
        *self.data_state.offset_mut() = new_offset;
        self.data_state.select(Some(new_offset));
    }

    pub fn previous(&mut self) {
        let len = match self.active_pane {
            ActivePane::Data => self.row_count(),
            ActivePane::Options => self.table.columns.len(),
        };
        if len == 0 {
            return;
        }
        let selected = match self.active_pane {
            ActivePane::Data => self.data_state.selected(),
            ActivePane::Options => self.options_state.selected(),
        };
        let i = match selected {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.select(Some(i));
    }
}
