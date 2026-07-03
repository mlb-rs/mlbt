use crate::components::constants::lookup_team_or;
use crate::components::date_selector::DateSelector;
use crate::components::datetime::format_game_time_padded;
use crate::components::decision_pitchers::GameDecisionPitchers;
use crate::components::probable_pitchers::{ProbablePitcher, ProbablePitcherMatchup};
use crate::components::standings::Team;
use crate::state::app_settings::AppSettings;
use crate::state::app_state::HomeOrAway;
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use core::option::Option::{None, Some};
use mlbt_api::schedule::{AbstractGameState, Game, LeagueRecord, ScheduleResponse};
use std::cmp::Ordering;
use tui::widgets::TableState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    GameStatus,
    Time,
}

/// ScheduleState is used to render the schedule as a `tui-rs` table.
pub struct ScheduleState {
    pub state: TableState,
    pub schedule: Vec<ScheduleRow>,
    pub date_selector: DateSelector,
    pub show_win_probability: bool,
    pub sort_mode: SortMode,
}

impl Default for ScheduleState {
    fn default() -> Self {
        ScheduleState {
            state: TableState::default(),
            schedule: Vec::new(),
            date_selector: DateSelector::default(),
            show_win_probability: true,
            sort_mode: SortMode::Time,
        }
    }
}

/// The information needed to create a single row in a table.
pub struct ScheduleRow {
    pub game_id: u64,
    pub home_team: Team,
    pub home_score: Option<u8>,
    pub home_record: Option<Record>,
    pub away_team: Team,
    pub away_score: Option<u8>,
    pub away_record: Option<Record>,
    /// Start time formatted for display in the user's current timezone.
    pub start_time: String,
    /// Used to rerender `start_time` when the configured timezone changes without refetching the
    /// schedule.
    pub start_time_utc: DateTime<Utc>,
    pub game_status: String,
    pub home_probable_pitcher: ProbablePitcher,
    pub away_probable_pitcher: ProbablePitcher,
    pub decision_pitchers: Option<GameDecisionPitchers>,
    pub abstract_game_state: Option<AbstractGameState>,
    pub current_inning: Option<i64>,
}

#[derive(Default, Copy, Clone)]
pub struct Record {
    pub wins: u8,
    pub losses: u8,
}

impl ScheduleState {
    /// Update the data from the API. It is assumed that the date is already updated, aka don't use
    /// a random date without first setting the `date` field. Use `set_date_from_input` for this.
    pub fn update(&mut self, settings: &AppSettings, schedule: &ScheduleResponse) {
        let selected_game_id = self.get_selected_game_opt();

        let rows = ScheduleRow::create_rows(settings, schedule);
        self.schedule = sort_schedule(rows, self.sort_mode, settings.favorite_team);

        // If schedule is empty, clear selection
        if self.is_empty() {
            self.state.select(None);
            return;
        }

        if self.date_selector.date_changed {
            self.date_selector.date_changed = false;
            self.state.select(Some(0));
        } else if let Some(new_index) = selected_game_id
            .and_then(|game_id| self.schedule.iter().position(|row| row.game_id == game_id))
        {
            // Re-find the previously selected game by id, since a re-sort (e.g. `GameStatus` mode
            // reacting to a game going live) can move it to a different index.
            self.state.select(Some(new_index));
        } else {
            self.state.select(Some(0));
        }
    }

    pub fn is_empty(&self) -> bool {
        self.schedule.is_empty()
    }

    /// Set the date from the validated input string from the date picker.
    pub fn set_date_from_valid_input(&mut self, date: NaiveDate) {
        self.date_selector.set_date_from_valid_input(date);
    }

    /// Set the date using Left/Right arrow keys to move a single day at a time.
    pub fn set_date_with_arrows(&mut self, forward: bool) -> NaiveDate {
        self.date_selector.set_date_with_arrows(forward)
    }

    /// Return the `game_id` of the row that is selected, or None if no row is selected.
    pub fn get_selected_game_opt(&self) -> Option<u64> {
        let idx = self.state.selected()?;
        self.schedule.get(idx).map(|s| s.game_id)
    }

    /// Return the probable pitchers for the selected game, or None if no game is selected.
    /// If the game is not in a "Scheduled" state, None will be returned.
    pub fn get_probable_pitchers_opt(&self) -> Option<ProbablePitcherMatchup<'_>> {
        let idx = self.state.selected()?;
        let row = self.schedule.get(idx)?;
        if row.game_status != "Scheduled" {
            return None;
        }
        Some(ProbablePitcherMatchup {
            home_pitcher: &row.home_probable_pitcher,
            home_team: row.home_team,
            away_pitcher: &row.away_probable_pitcher,
            away_team: row.away_team,
        })
    }

    /// Look up decisions by `game_id` so the right panel can load them based on the gameday-loaded
    /// game, keeping them in sync with the linescore and box score.
    pub fn get_decision_pitchers_for_game(&self, game_id: u64) -> Option<&GameDecisionPitchers> {
        if game_id == 0 {
            return None;
        }
        let row = self.schedule.iter().find(|row| row.game_id == game_id)?;
        row.decision_pitchers.as_ref()
    }

    pub fn toggle_win_probability(&mut self) {
        self.show_win_probability = !self.show_win_probability;
    }

    /// Re-render every row's `start_time` string using the given timezone.
    /// Called after the user changes timezone so times update without a schedule refetch.
    pub fn refresh_start_times(&mut self, tz: Tz) {
        for row in &mut self.schedule {
            row.start_time = format_game_time_padded(row.start_time_utc, tz);
        }
    }

    /// Reorder the already loaded rows to reflect the current favorite team. Selects the first row
    /// so the Scoreboard jumps to the favorite's game (or the top of the list when no favorite is
    /// set). Callers should refetch game data when this shifts the selection.
    pub fn apply_favorite_team(&mut self, favorite_team: Option<Team>) {
        let rows = std::mem::take(&mut self.schedule);
        self.schedule = sort_schedule(rows, self.sort_mode, favorite_team);

        if self.schedule.is_empty() {
            self.state.select(None);
            return;
        }

        self.state.select(Some(0));
    }

    pub fn toggle_sort_mode(&mut self, favorite_team: Option<Team>) {
        self.sort_mode = match self.sort_mode {
            SortMode::GameStatus => SortMode::Time,
            SortMode::Time => SortMode::GameStatus,
        };
        let rows = std::mem::take(&mut self.schedule);
        self.schedule = sort_schedule(rows, self.sort_mode, favorite_team);
        self.state.select(Some(0));
    }

    pub fn next(&mut self) {
        if self.schedule.is_empty() {
            self.state.select(None);
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.schedule.len() - 1 {
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
        if self.schedule.is_empty() {
            self.state.select(None);
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.schedule.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl Record {
    pub fn from_league_record(record: Option<&LeagueRecord>) -> Option<Self> {
        record.map(|r| Self {
            wins: r.wins,
            losses: r.losses,
        })
    }

    pub fn to_display_string(self) -> String {
        format!("{}-{}", self.wins, self.losses)
    }

    pub fn default_display_string() -> String {
        "--".to_string()
    }
}

impl ScheduleRow {
    /// Determine which team, if either, is winning the game. A team may not have a score, e.g. if
    /// the game hasn't started yet, or the game may be tied, so return None in these cases.
    pub fn winning_team(&self) -> Option<HomeOrAway> {
        let home = self.home_score?;
        let away = self.away_score?;

        match home.cmp(&away) {
            Ordering::Greater => Some(HomeOrAway::Home),
            Ordering::Less => Some(HomeOrAway::Away),
            Ordering::Equal => None,
        }
    }

    /// Create the matchup information to be displayed in the table. The current information that is
    /// extracted from the game data:
    /// away team name and score, home team name and score, start time, and game status
    fn create_matchup(game: &Game, timezone: Tz) -> Self {
        let home_team = &game.teams.home;
        let home_name = lookup_team_or(&home_team.team.name, || {
            Team::from_schedule(&home_team.team)
        });
        let home_record = Record::from_league_record(home_team.league_record.as_ref());

        let away_team = &game.teams.away;
        let away_name = lookup_team_or(&away_team.team.name, || {
            Team::from_schedule(&away_team.team)
        });
        let away_record = Record::from_league_record(away_team.league_record.as_ref());

        let start_time_utc = game.game_date;
        let start_time = format_game_time_padded(start_time_utc, timezone);

        let game_status = match &game.status.detailed_state {
            Some(s) if s == "In Progress" => {
                if let Some(linescore) = game.linescore.as_ref() {
                    let half = match linescore.is_top_inning.unwrap_or(true) {
                        true => "Top",
                        false => "Bottom",
                    };
                    format!(
                        "{half} {}",
                        linescore.current_inning_ordinal.as_deref().unwrap_or("1st")
                    )
                } else {
                    s.clone()
                }
            }
            Some(s) => s.clone(),
            None => "-".to_string(),
        };

        ScheduleRow {
            game_id: game.game_pk,
            home_team: home_name,
            home_record,
            home_score: game.teams.home.score,
            away_record,
            away_team: away_name,
            away_score: game.teams.away.score,
            game_status,
            start_time,
            start_time_utc,
            home_probable_pitcher: ProbablePitcher::from_team(&game.teams.home).unwrap_or_default(),
            away_probable_pitcher: ProbablePitcher::from_team(&game.teams.away).unwrap_or_default(),
            decision_pitchers: GameDecisionPitchers::from_game(game),
            abstract_game_state: game.status.abstract_game_state,
            current_inning: game.linescore.as_ref().and_then(|l| l.current_inning),
        }
    }

    /// Transform the data from the API into a vector of ScheduleRows.
    fn create_rows(settings: &AppSettings, schedule: &ScheduleResponse) -> Vec<Self> {
        let mut todays_games: Vec<ScheduleRow> = Vec::with_capacity(schedule.dates.len());
        if let Some(games) = &schedule.dates.first()
            && let Some(game) = &games.games
        {
            for g in game {
                todays_games.push(ScheduleRow::create_matchup(g, settings.timezone));
            }
        }
        todays_games
    }

    fn has_team(&self, team: Team) -> bool {
        self.home_team.id == team.id || self.away_team.id == team.id
    }
}

fn sort_time(
    rows: impl IntoIterator<Item = ScheduleRow>,
    favorite_team: Option<Team>,
) -> Vec<ScheduleRow> {
    let mut favorite = Vec::new();
    let mut other = Vec::new();

    for row in rows {
        if favorite_team.is_some_and(|team| row.has_team(team)) {
            favorite.push(row);
        } else {
            other.push(row);
        }
    }

    // Break start-time ties by game id so the order matches `sort_game_status` and stays stable
    // across re-sorts, rather than depending on the order the API returned games in.
    favorite.sort_by_key(|row| (row.start_time_utc, row.game_id));
    other.sort_by_key(|row| (row.start_time_utc, row.game_id));
    favorite.extend(other);
    favorite
}

fn sort_game_status(
    rows: impl IntoIterator<Item = ScheduleRow>,
    favorite_team: Option<Team>,
) -> Vec<ScheduleRow> {
    fn category(row: &ScheduleRow) -> u8 {
        match row.abstract_game_state {
            Some(AbstractGameState::Live) => 0,
            Some(AbstractGameState::Preview) | Some(AbstractGameState::Other) | None => 1,
            Some(AbstractGameState::Final) => 2,
        }
    }

    let mut rows: Vec<ScheduleRow> = rows.into_iter().collect();
    rows.sort_by(|a, b| {
        let a_fav = favorite_team.is_some_and(|t| a.has_team(t));
        let b_fav = favorite_team.is_some_and(|t| b.has_team(t));

        if a_fav != b_fav {
            return if a_fav {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }

        let a_cat = category(a);
        let b_cat = category(b);
        if a_cat != b_cat {
            return a_cat.cmp(&b_cat);
        }

        match a_cat {
            0 => b
                .current_inning
                .unwrap_or(0)
                .cmp(&a.current_inning.unwrap_or(0))
                .then(a.game_id.cmp(&b.game_id)),
            1 | 2 => a
                .start_time_utc
                .cmp(&b.start_time_utc)
                .then(a.game_id.cmp(&b.game_id)),
            _ => Ordering::Equal,
        }
    });
    rows
}

fn sort_schedule(
    rows: Vec<ScheduleRow>,
    mode: SortMode,
    favorite_team: Option<Team>,
) -> Vec<ScheduleRow> {
    match mode {
        SortMode::GameStatus => sort_game_status(rows, favorite_team),
        SortMode::Time => sort_time(rows, favorite_team),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::constants::lookup_team_by_id;
    use crate::components::probable_pitchers::ProbablePitcher;

    fn row(game_id: u64, home_team: u16, away_team: u16) -> ScheduleRow {
        ScheduleRow {
            game_id,
            home_team: lookup_team_by_id(home_team).unwrap(),
            home_score: None,
            home_record: None,
            away_team: lookup_team_by_id(away_team).unwrap(),
            away_score: None,
            away_record: None,
            start_time: String::new(),
            start_time_utc: DateTime::from_timestamp(game_id as i64, 0).unwrap(),
            game_status: String::new(),
            home_probable_pitcher: ProbablePitcher::default(),
            away_probable_pitcher: ProbablePitcher::default(),
            decision_pitchers: None,
            abstract_game_state: None,
            current_inning: None,
        }
    }

    #[test]
    fn apply_favorite_team_reorders_rows_and_resets_selection() {
        let mut state = ScheduleState {
            state: TableState::default(),
            schedule: vec![row(30, 114, 115), row(10, 108, 109), row(20, 112, 113)],
            date_selector: DateSelector::default(),
            show_win_probability: true,
            sort_mode: SortMode::Time,
        };
        state.state.select(Some(2));

        state.apply_favorite_team(lookup_team_by_id(112));

        assert_eq!(
            state
                .schedule
                .iter()
                .map(|row| row.game_id)
                .collect::<Vec<_>>(),
            vec![20, 10, 30]
        );
        assert_eq!(state.get_selected_game_opt(), Some(20));
        assert_eq!(state.state.selected(), Some(0));
    }

    #[test]
    fn apply_favorite_team_none_sorts_by_game_id_and_resets_selection() {
        let mut state = ScheduleState {
            state: TableState::default(),
            schedule: vec![row(30, 114, 115), row(10, 108, 109), row(20, 112, 113)],
            date_selector: DateSelector::default(),
            show_win_probability: true,
            sort_mode: SortMode::Time,
        };
        state.state.select(Some(0));

        state.apply_favorite_team(None);

        assert_eq!(
            state
                .schedule
                .iter()
                .map(|row| row.game_id)
                .collect::<Vec<_>>(),
            vec![10, 20, 30]
        );
        assert_eq!(state.get_selected_game_opt(), Some(10));
        assert_eq!(state.state.selected(), Some(0));
    }

    fn row_with_state(
        game_id: u64,
        home_team: u16,
        away_team: u16,
        state: Option<AbstractGameState>,
        inning: Option<i64>,
        start_time: DateTime<Utc>,
    ) -> ScheduleRow {
        ScheduleRow {
            game_id,
            home_team: lookup_team_by_id(home_team).unwrap(),
            home_score: None,
            home_record: None,
            away_team: lookup_team_by_id(away_team).unwrap(),
            away_score: None,
            away_record: None,
            start_time: String::new(),
            start_time_utc: start_time,
            game_status: String::new(),
            home_probable_pitcher: ProbablePitcher::default(),
            away_probable_pitcher: ProbablePitcher::default(),
            decision_pitchers: None,
            abstract_game_state: state,
            current_inning: inning,
        }
    }

    fn ts(secs: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(secs, 0).unwrap()
    }

    fn ids(rows: Vec<ScheduleRow>) -> Vec<u64> {
        rows.iter().map(|r| r.game_id).collect()
    }

    #[test]
    fn sort_game_status_orders_by_favorite_then_category() {
        // Favorite team is pinned first even when its game is Final, then live games, then the
        // upcoming bucket (Preview/Other/None) by start time, then remaining finished games.
        let rows = vec![
            row_with_state(1, 108, 109, Some(AbstractGameState::Final), None, ts(0)),
            row_with_state(2, 112, 113, Some(AbstractGameState::Final), None, ts(0)),
            row_with_state(3, 114, 115, Some(AbstractGameState::Live), Some(5), ts(0)),
            row_with_state(4, 116, 117, Some(AbstractGameState::Preview), None, ts(10)),
            row_with_state(5, 118, 119, Some(AbstractGameState::Other), None, ts(20)),
            row_with_state(6, 120, 121, None, None, ts(30)),
        ];
        let sorted = sort_game_status(rows, lookup_team_by_id(112));
        assert_eq!(ids(sorted), vec![2, 3, 4, 5, 6, 1]);
    }

    #[test]
    fn sort_game_status_tiebreaks_within_category() {
        // Live games sort by current inning descending.
        let live = vec![
            row_with_state(1, 108, 109, Some(AbstractGameState::Live), Some(3), ts(0)),
            row_with_state(2, 112, 113, Some(AbstractGameState::Live), Some(7), ts(0)),
            row_with_state(3, 114, 115, Some(AbstractGameState::Live), Some(1), ts(0)),
        ];
        assert_eq!(ids(sort_game_status(live, None)), vec![2, 1, 3]);

        // Finished games sort by game id ascending.
        let finished = vec![
            row_with_state(30, 114, 115, Some(AbstractGameState::Final), None, ts(0)),
            row_with_state(10, 108, 109, Some(AbstractGameState::Final), None, ts(0)),
            row_with_state(20, 112, 113, Some(AbstractGameState::Final), None, ts(0)),
        ];
        assert_eq!(ids(sort_game_status(finished, None)), vec![10, 20, 30]);
    }

    #[test]
    fn sort_time_orders_by_start_time_then_game_id_favorite_first() {
        // Games 3 and 4 share a start time and are given in reverse game-id order to prove ties
        // break by game id (matching sort_game_status), not by input/API order.
        let rows = vec![
            row_with_state(1, 108, 109, None, None, ts(300)),
            row_with_state(4, 116, 117, None, None, ts(200)),
            row_with_state(2, 112, 113, None, None, ts(100)),
            row_with_state(3, 114, 115, None, None, ts(200)),
        ];
        assert_eq!(ids(sort_time(rows, None)), vec![2, 3, 4, 1]);

        let rows = vec![
            row_with_state(1, 108, 109, None, None, ts(300)),
            row_with_state(2, 112, 113, None, None, ts(100)),
            row_with_state(3, 114, 115, None, None, ts(200)),
        ];
        // Favorite (team 108, game 1) is pinned first despite its latest start time.
        assert_eq!(ids(sort_time(rows, lookup_team_by_id(108))), vec![1, 2, 3]);
    }

    #[test]
    fn toggle_sort_mode_switches_and_reselects() {
        let mut state = ScheduleState {
            state: TableState::default(),
            schedule: vec![row(30, 114, 115), row(10, 108, 109), row(20, 112, 113)],
            date_selector: DateSelector::default(),
            show_win_probability: true,
            sort_mode: SortMode::Time,
        };
        state.state.select(Some(1));
        state.toggle_sort_mode(lookup_team_by_id(112));
        assert_eq!(state.sort_mode, SortMode::GameStatus);
        assert_eq!(state.state.selected(), Some(0));
    }
}
