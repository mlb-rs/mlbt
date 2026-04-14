use crate::components::constants::lookup_team_or;
use crate::components::date_selector::DateSelector;
use crate::components::probable_pitchers::{ProbablePitcher, ProbablePitcherMatchup};
use crate::components::standings::Team;
use crate::state::app_settings::AppSettings;
use crate::state::app_state::HomeOrAway;
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use core::option::Option::{None, Some};
use log::error;
use mlbt_api::schedule::{Game, LeagueRecord, ScheduleResponse};
use std::cmp::Ordering;
use tui::widgets::TableState;

/// ScheduleState is used to render the schedule as a `tui-rs` table.
pub struct ScheduleState {
    pub state: TableState,
    pub schedule: Vec<ScheduleRow>,
    pub date_selector: DateSelector,
    pub show_win_probability: bool,
}

impl Default for ScheduleState {
    fn default() -> Self {
        ScheduleState {
            state: TableState::default(),
            schedule: Vec::new(),
            date_selector: DateSelector::default(),
            show_win_probability: true,
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
        self.schedule = ScheduleRow::create_table(settings, schedule);

        // If schedule is empty, clear selection
        if self.is_empty() {
            self.state.select(None);
            return;
        }

        if self.date_selector.date_changed {
            self.date_selector.date_changed = false;
            self.state.select(Some(0));
        } else if self.state.selected().is_none() {
            self.state.select(Some(0));
        } else if let Some(selected) = self.state.selected()
            && selected >= self.schedule.len()
        {
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

    pub fn toggle_win_probability(&mut self) {
        self.show_win_probability = !self.show_win_probability;
    }

    /// Re-render every row's `start_time` string using the given timezone.
    /// Called after the user changes timezone so times update without a schedule refetch.
    pub fn refresh_start_times(&mut self, tz: Tz) {
        for row in &mut self.schedule {
            row.start_time = format_start_time(row.start_time_utc, tz);
        }
    }

    /// Reorder the already loaded rows to reflect the current favorite team while preserving the
    /// selected game.
    pub fn apply_favorite_team(&mut self, favorite_team: Option<Team>) {
        let rows = std::mem::take(&mut self.schedule);
        self.schedule = favorite_first(rows, favorite_team);

        if self.schedule.is_empty() {
            self.state.select(None);
            return;
        }

        self.state.select(Some(0));
    }

    pub fn next(&mut self) {
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

        let start_time_utc = DateTime::parse_from_rfc3339(&game.game_date)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|err| {
                error!("invalid game_date {:?}: {err}", game.game_date);
                DateTime::<Utc>::UNIX_EPOCH
            });
        let start_time = format_start_time(start_time_utc, timezone);

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
        }
    }

    /// Transform the data from the API into a vector of ScheduleRows.
    fn create_table(settings: &AppSettings, schedule: &ScheduleResponse) -> Vec<Self> {
        let mut todays_games: Vec<ScheduleRow> = Vec::with_capacity(schedule.dates.len());
        if let Some(games) = &schedule.dates.first() {
            if let Some(game) = &games.games {
                for g in game {
                    todays_games.push(ScheduleRow::create_matchup(g, settings.timezone));
                }
            }
        }
        favorite_first(todays_games, settings.favorite_team)
    }

    fn has_team(&self, team: Team) -> bool {
        self.home_team.id == team.id || self.away_team.id == team.id
    }
}

fn favorite_first(
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

    favorite.sort_by_key(|row| row.game_id);
    other.sort_by_key(|row| row.game_id);
    favorite.extend(other);
    favorite
}

/// Format a UTC game start time for display in the given timezone (e.g. "7:05 pm").
fn format_start_time(utc: DateTime<Utc>, tz: Tz) -> String {
    utc.with_timezone(&tz).format("%l:%M %P").to_string()
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
            start_time_utc: DateTime::<Utc>::UNIX_EPOCH,
            game_status: String::new(),
            home_probable_pitcher: ProbablePitcher::default(),
            away_probable_pitcher: ProbablePitcher::default(),
        }
    }

    #[test]
    fn apply_favorite_team_reorders_rows_and_resets_selection() {
        let mut state = ScheduleState {
            state: TableState::default(),
            schedule: vec![row(30, 114, 115), row(10, 108, 109), row(20, 112, 113)],
            date_selector: DateSelector::default(),
            show_win_probability: true,
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
}
