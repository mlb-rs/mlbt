use crate::config::TomlFileStore;
use crate::state::app_settings::AppSettings;
use crate::state::app_state::AppState;
use crate::state::settings_editor::SettingsStatus;
use chrono::{NaiveDate, ParseError, Utc};
use log::{error, info};
use mlbt_api::live::LiveResponse;
use mlbt_api::player::PeopleResponse;
use mlbt_api::schedule::ScheduleResponse;
use mlbt_api::season::GameType;
use mlbt_api::team::{RosterResponse, RosterType, TransactionsResponse};
use mlbt_api::win_probability::WinProbabilityResponse;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum MenuItem {
    #[default]
    Scoreboard,
    Gameday,
    Stats,
    Standings,
    Help,
    DatePicker,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum DebugState {
    On,
    #[default]
    Off,
}

pub struct App {
    pub settings: AppSettings,
    pub state: AppState,
    store: TomlFileStore,
}

impl App {
    pub fn new() -> Self {
        let store = TomlFileStore::default();
        let settings = AppSettings::load(&store);
        let mut app = Self {
            state: AppState::default(),
            settings,
            store,
        };
        app.configure();
        app
    }

    /// Startup only configuration that should run once after settings are loaded.
    fn configure(&mut self) {
        self.set_all_datepickers_to_today();
        self.state.standings.favorite_team = self.settings.favorite_team;
        self.apply_log_level();
    }

    /// Replay only the side effects driven by settings that changed at runtime.
    fn apply_runtime_settings(&mut self, previous: &AppSettings) {
        if self.settings.favorite_team.map(|t| t.id) != previous.favorite_team.map(|t| t.id) {
            self.state
                .schedule
                .apply_favorite_team(self.settings.favorite_team);
            self.state
                .standings
                .apply_favorite_team(self.settings.favorite_team);
        }

        if self.settings.timezone != previous.timezone {
            self.state
                .schedule
                .refresh_start_times(self.settings.timezone);
            if let Some(tp) = &mut self.state.standings.team_page {
                tp.refresh_schedule_times(self.settings.timezone);
            }
            if let Some(tp) = &mut self.state.stats.team_page {
                tp.refresh_schedule_times(self.settings.timezone);
            }
        }

        if self.settings.log_level != previous.log_level {
            self.apply_log_level();
        }
    }

    /// Commit the picker's current selection: apply to settings, persist, and replay settings
    /// driven side effects. Sets a status (Saved / Error) on the settings editor, but keeps the in
    /// memory change even on save failure.
    ///
    /// Returns whether the runtime side effects shifted the Scoreboard's selected game (e.g.
    /// changing a favorite team), so the caller can refetch game data to keep the box score in sync
    /// with the Scoreboard selection.
    pub fn commit_settings_picker(&mut self) -> bool {
        let Some(picker) = self.state.settings_editor.picker.clone() else {
            return false;
        };
        let previous_settings = self.settings.clone();
        let previous_game_id = self.state.schedule.get_selected_game_opt();

        picker.field.apply(picker.cursor, &mut self.settings);
        self.state.settings_editor.close_picker();
        self.apply_runtime_settings(&previous_settings);

        let status = match self.store.save(&self.settings) {
            Ok(()) => {
                info!("settings saved");
                SettingsStatus::Saved
            }
            Err(err) => {
                error!("could not save settings: {err}");
                SettingsStatus::Error(err.to_string())
            }
        };
        self.state.settings_editor.status = Some(status);

        let new_game_id = self.state.schedule.get_selected_game_opt();
        new_game_id != previous_game_id && new_game_id.is_some()
    }

    fn apply_log_level(&self) {
        let level = log::LevelFilter::from(self.settings.log_level);
        log::set_max_level(level);
        tui_logger::set_default_level(level);
    }

    /// Sync date pickers using the correct timezone.
    fn set_all_datepickers_to_today(&mut self) {
        let today = Utc::now()
            .with_timezone(&self.settings.timezone)
            .date_naive();
        self.state.schedule.date_selector.date = today;
        self.state.stats.date_selector.date = today;
        self.state.standings.date_selector.date = today;
    }

    /// Update the schedule and return the selected game.
    /// If the schedule is empty, return None.
    pub fn update_schedule(&mut self, schedule: &ScheduleResponse) -> Option<u64> {
        let old_game_id = self.state.gameday.current_game_id();
        self.state.schedule.update(&self.settings, schedule);
        let selected = self.state.schedule.get_selected_game_opt();

        // reset data based on the currently selected game
        self.state.gameday.reset(selected);
        self.state.box_score.reset(selected);

        // return the game id only if it changed
        match selected {
            Some(new_id) if new_id != old_game_id && new_id > 0 => Some(new_id),
            _ => None,
        }
    }

    pub fn update_live_data(
        &mut self,
        live_data: &LiveResponse,
        win_probability: &WinProbabilityResponse,
    ) {
        // only update gameday if the selected game is the same as the game being updated
        // this prevents gameday from showing incorrect data if the user scrolls through games quickly
        if Some(live_data.game_pk) == self.state.schedule.get_selected_game_opt() {
            self.state.gameday.game.update(live_data, win_probability);
            // update this after the gameday so the players are correct
            self.state
                .box_score
                .update(live_data, &self.state.gameday.game.players);

            // only reset the scroll state if on the scoreboard tab. this will reset when a new game
            // is selected or the data refreshes. don't reset the scroll in Gameday because that
            // happens too frequently and makes it hard to read the box score
            if self.state.active_tab == MenuItem::Scoreboard {
                self.state.box_score.reset_scroll();
            }
        }
    }

    pub fn update_tab(&mut self, next: MenuItem) {
        // don't switch tabs if already on the correct tab
        if self.state.active_tab == next {
            return;
        }

        self.state.previous_tab = self.state.active_tab;
        self.state.active_tab = next;
        self.state.debug_state = DebugState::Off;

        if self.state.previous_tab == MenuItem::Help {
            self.state.settings_editor.status = None;
        }

        // reset help state when switching tabs but not when its opened/closed on same tab
        if next != MenuItem::Help {
            self.state.help.reset();
        }

        // reset standings selection when switching tabs but not when date picker is opened
        if next != MenuItem::DatePicker && self.state.previous_tab == MenuItem::Standings {
            self.state.standings.reset_selection();
        }

        // reset boxscore scroll
        if next != MenuItem::DatePicker
            && (self.state.previous_tab == MenuItem::Scoreboard
                || self.state.previous_tab == MenuItem::Gameday)
        {
            self.state.box_score.reset_scroll();
        }
    }

    pub fn try_update_date_from_input(&mut self) -> Result<(), ParseError> {
        let valid_date = self
            .state
            .date_input
            .validate_input(self.settings.timezone)?;

        // current tab is date picker, so use previous tab to update correct date
        match self.state.previous_tab {
            MenuItem::Scoreboard => self.state.schedule.set_date_from_valid_input(valid_date),
            MenuItem::Standings => self.state.standings.set_date_from_valid_input(valid_date),
            MenuItem::Stats => self.state.stats.set_date_from_valid_input(valid_date),
            _ => (),
        }
        Ok(())
    }

    pub fn move_date_selector_by_arrow(&mut self, right_arrow: bool) {
        let date = match self.state.previous_tab {
            MenuItem::Scoreboard => Some(self.state.schedule.set_date_with_arrows(right_arrow)),
            MenuItem::Standings => Some(self.state.standings.set_date_with_arrows(right_arrow)),
            MenuItem::Stats => Some(self.state.stats.set_date_with_arrows(right_arrow)),
            _ => None,
        };
        self.state.date_input.text.clear();
        if let Some(date) = date {
            self.state.date_input.text.push_str(&date.to_string());
        }
    }

    pub fn exit_help(&mut self) {
        if self.state.active_tab == MenuItem::Help {
            self.state.active_tab = self.state.previous_tab;
            self.state.settings_editor.status = None;
        }
    }

    pub fn toggle_debug(&mut self) {
        match self.state.debug_state {
            DebugState::Off => self.state.debug_state = DebugState::On,
            DebugState::On => self.state.debug_state = DebugState::Off,
        }
    }

    pub fn toggle_show_logs(&mut self) {
        self.state.show_logs = !self.state.show_logs;
    }

    pub fn toggle_full_screen(&mut self) {
        self.settings.full_screen = !self.settings.full_screen;
    }

    pub fn update_player_profile(&mut self, data: Arc<PeopleResponse>, game_type: GameType) {
        match self.state.active_tab {
            MenuItem::Standings if self.state.standings.has_team_page() => {
                self.state
                    .standings
                    .update_team_player_profile(data, game_type);
            }
            MenuItem::Stats if self.state.stats.has_team_page() => {
                self.state.stats.update_team_player_profile(data, game_type);
            }
            MenuItem::Stats => {
                self.state.stats.update_player_profile(data, game_type);
            }
            _ => {}
        }
    }

    pub fn update_team_page(
        &mut self,
        team_id: u16,
        date: NaiveDate,
        schedule: &ScheduleResponse,
        roster: &RosterResponse,
        transactions: &TransactionsResponse,
    ) {
        let tz = self.settings.timezone;
        match self.state.active_tab {
            MenuItem::Standings => {
                self.state.standings.update_team_page(
                    team_id,
                    date,
                    schedule,
                    roster,
                    transactions,
                    tz,
                );
            }
            MenuItem::Stats => {
                self.state.stats.update_team_page(
                    team_id,
                    date,
                    schedule,
                    roster,
                    transactions,
                    tz,
                );
            }
            _ => {}
        }
    }

    pub fn close_overlay(&mut self) {
        match self.state.active_tab {
            MenuItem::Standings => self.state.standings.close_overlay(),
            MenuItem::Stats => self.state.stats.close_overlay(),
            _ => {}
        }
    }

    pub fn update_team_roster(
        &mut self,
        team_id: u16,
        roster: &RosterResponse,
        roster_type: RosterType,
    ) {
        match self.state.active_tab {
            MenuItem::Standings => {
                self.state
                    .standings
                    .update_team_roster(team_id, roster, roster_type);
            }
            MenuItem::Stats => {
                self.state
                    .stats
                    .update_team_roster(team_id, roster, roster_type);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::constants::lookup_team_by_id;
    use crate::components::probable_pitchers::ProbablePitcher;
    use crate::components::schedule::ScheduleRow;
    use crate::components::team_page::TeamGame;
    use crate::config::ConfigFile;
    use crate::state::settings_editor::{PickerState, SettingsField, TEAM_OPTIONS};
    use crate::state::team_page::{TeamPageState, TeamSection};
    use tui::widgets::TableState;

    fn test_app() -> App {
        App {
            settings: AppSettings::from(ConfigFile::default()),
            state: AppState::default(),
            store: TomlFileStore::default(),
        }
    }

    fn test_team_page(time_or_score: &str) -> TeamPageState {
        TeamPageState {
            team: lookup_team_by_id(112).unwrap(),
            date: NaiveDate::from_ymd_opt(2025, 3, 28).unwrap(),
            schedule: vec![TeamGame {
                date: NaiveDate::from_ymd_opt(2025, 3, 28).unwrap(),
                date_display: "Mar 28".to_string(),
                opponent: "@ AZ".to_string(),
                time_or_score: time_or_score.to_string(),
                start_time_utc: Some(
                    NaiveDate::from_ymd_opt(2025, 3, 28)
                        .unwrap()
                        .and_hms_opt(23, 10, 0)
                        .unwrap()
                        .and_utc(),
                ),
                is_home: false,
                is_past: false,
                is_win: None,
            }],
            schedule_selection: TableState::default(),
            roster: vec![],
            roster_type: RosterType::Active,
            transactions: vec![],
            selected_transaction: 0,
            transaction_scroll: 0,
            active_section: TeamSection::Roster,
            roster_selection: TableState::default(),
            roster_table_len: 0,
            roster_header_rows: std::collections::HashSet::new(),
            roster_row_map: vec![],
            player_profile: None,
            show_calendar: true,
        }
    }

    fn test_schedule_row(game_id: u64, home_team: u16, away_team: u16) -> ScheduleRow {
        ScheduleRow {
            game_id,
            home_team: lookup_team_by_id(home_team).unwrap(),
            home_score: None,
            home_record: None,
            away_team: lookup_team_by_id(away_team).unwrap(),
            away_score: None,
            away_record: None,
            start_time: String::new(),
            start_time_utc: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
            game_status: String::new(),
            home_probable_pitcher: ProbablePitcher::default(),
            away_probable_pitcher: ProbablePitcher::default(),
        }
    }

    #[test]
    fn runtime_settings_change_preserves_selected_dates() {
        let mut app = test_app();
        let historical_date = NaiveDate::from_ymd_opt(2024, 7, 4).unwrap();
        app.state.schedule.date_selector.date = historical_date;
        app.state.stats.date_selector.date = historical_date;
        app.state.standings.date_selector.date = historical_date;

        let previous_settings = app.settings.clone();
        app.settings.favorite_team = lookup_team_by_id(112);
        app.apply_runtime_settings(&previous_settings);

        assert_eq!(app.state.schedule.date_selector.date, historical_date);
        assert_eq!(app.state.stats.date_selector.date, historical_date);
        assert_eq!(app.state.standings.date_selector.date, historical_date);
    }

    #[test]
    fn runtime_timezone_change_refreshes_open_team_pages() {
        let mut app = test_app();
        app.settings.timezone = chrono_tz::US::Eastern;
        app.state.standings.team_page = Some(test_team_page("7:10 PM"));
        app.state.stats.team_page = Some(test_team_page("7:10 PM"));

        let previous_settings = app.settings.clone();
        app.settings.timezone = chrono_tz::US::Pacific;
        app.apply_runtime_settings(&previous_settings);

        assert_eq!(
            app.state
                .standings
                .team_page
                .as_ref()
                .unwrap()
                .schedule
                .first()
                .unwrap()
                .time_or_score,
            "4:10 pm"
        );
        assert_eq!(
            app.state
                .stats
                .team_page
                .as_ref()
                .unwrap()
                .schedule
                .first()
                .unwrap()
                .time_or_score,
            "4:10 pm"
        );
    }

    #[test]
    fn commit_settings_picker_reports_when_selected_game_changes() {
        let mut app = test_app();
        let cubs_index = TEAM_OPTIONS
            .iter()
            .position(|team| team.map(|t| t.id) == Some(112))
            .unwrap();
        app.state.schedule.schedule = vec![
            test_schedule_row(30, 114, 115),
            test_schedule_row(10, 108, 109),
            test_schedule_row(20, 112, 113),
        ];
        app.state.schedule.state.select(Some(0));
        app.state.settings_editor.picker = Some(PickerState {
            field: SettingsField::FavoriteTeam,
            cursor: cubs_index,
        });

        let game_id_changed = app.commit_settings_picker();

        assert!(game_id_changed);
        assert_eq!(app.state.schedule.get_selected_game_opt(), Some(20));
    }

    #[test]
    fn exit_help_clears_settings_status() {
        let mut app = test_app();
        app.state.active_tab = MenuItem::Help;
        app.state.previous_tab = MenuItem::Scoreboard;
        app.state.settings_editor.status = Some(SettingsStatus::Saved);

        app.exit_help();

        assert_eq!(app.state.active_tab, MenuItem::Scoreboard);
        assert!(app.state.settings_editor.status.is_none());
    }

    #[test]
    fn leaving_help_via_tab_switch_clears_settings_status() {
        let mut app = test_app();
        app.state.active_tab = MenuItem::Help;
        app.state.previous_tab = MenuItem::Scoreboard;
        app.state.settings_editor.status = Some(SettingsStatus::Saved);

        app.update_tab(MenuItem::Stats);

        assert_eq!(app.state.active_tab, MenuItem::Stats);
        assert!(app.state.settings_editor.status.is_none());
    }
}
