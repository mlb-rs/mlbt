use crate::app::{AppSettings, HomeOrAway};
use crate::components::constants::TEAM_NAMES;
use crate::components::date_selector::DateSelector;
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use core::option::Option::{None, Some};
use mlb_api::schedule::{Game, ScheduleResponse};
use std::cmp::Ordering;
use tui::widgets::TableState;

/// ScheduleState is used to render the schedule as a `tui-rs` table.
#[derive(Default)]
pub struct ScheduleState {
    pub state: TableState,
    pub schedule: Vec<ScheduleRow>,
    pub date_selector: DateSelector,
}

/// The information needed to create a single row in a table.
#[derive(Default)]
pub struct ScheduleRow {
    pub game_id: u64,
    pub home_team: String,
    pub home_score: Option<u8>,
    pub away_team: String,
    pub away_score: Option<u8>,
    pub start_time: String,
    pub game_status: String,
}

impl ScheduleState {
    pub fn from_schedule(settings: &AppSettings, schedule: &ScheduleResponse) -> Self {
        let date_selector = DateSelector::new(ScheduleRow::get_date_from_schedule(schedule));
        let mut ss = ScheduleState {
            state: TableState::default(),
            schedule: ScheduleRow::create_table(settings, schedule),
            date_selector,
        };
        ss.state.select(Some(0));
        ss
    }

    /// Update the data from the API. It is assumed that the date is already updated, aka don't use
    /// a random date without first setting the `date` field. Use `set_date_from_input` for this.
    pub fn update(&mut self, settings: &AppSettings, schedule: &ScheduleResponse) {
        self.schedule = ScheduleRow::create_table(settings, schedule);
    }

    /// Set the date from the validated input string from the date picker.
    pub fn set_date_from_valid_input(&mut self, date: NaiveDate) {
        self.date_selector.set_date_from_valid_input(date);
        self.state.select(Some(0));
    }

    /// Set the date using Left/Right arrow keys to move a single day at a time.
    pub fn set_date_with_arrows(&mut self, forward: bool) -> NaiveDate {
        self.date_selector.set_date_with_arrows(forward)
    }

    /// Return the `game_id` of the row that is selected.
    pub fn get_selected_game(&self) -> u64 {
        let idx = match self.state.selected() {
            Some(s) => s,
            None => return 0,
        };
        match self.schedule.get(idx) {
            Some(s) => s.game_id,
            _ => 0,
        }
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
        let home_team = TEAM_NAMES
            .get(&*game.teams.home.team.name)
            .unwrap_or(&"unknown")
            .to_string();

        let away_team = TEAM_NAMES
            .get(&*game.teams.away.team.name)
            .unwrap_or(&"unknown")
            .to_string();

        let datetime = DateTime::parse_from_rfc3339(&game.game_date)
            .unwrap()
            .with_timezone(&timezone);
        let start_time = datetime.format("%l:%M %P").to_string();

        let game_status = match &game.status.detailed_state {
            Some(s) => s.to_string(),
            _ => "-".to_string(),
        };

        ScheduleRow {
            game_id: game.game_pk,
            home_team,
            home_score: game.teams.home.score,
            away_team,
            away_score: game.teams.away.score,
            game_status,
            start_time,
        }
    }

    /// Transform the data from the API into a vector of ScheduleRows.
    fn create_table(settings: &AppSettings, schedule: &ScheduleResponse) -> Vec<Self> {
        let mut todays_games: Vec<ScheduleRow> = Vec::with_capacity(schedule.dates.len());
        if let Some(games) = &schedule.dates.first() {
            let favorite = settings
                .favorite_team
                .clone()
                .unwrap_or_else(|| "na".to_string());
            if let Some(game) = &games.games {
                for g in game {
                    let row = ScheduleRow::create_matchup(g, settings.timezone);
                    if g.teams.home.team.name == favorite || g.teams.away.team.name == favorite {
                        todays_games.insert(0, row);
                    } else {
                        todays_games.push(row);
                    }
                }
            }
        }
        todays_games
    }

    /// The date is stored in schedule -> dates -> date.
    fn get_date_from_schedule(schedule: &ScheduleResponse) -> NaiveDate {
        let now = Utc::now().naive_local().date();
        if let Some(games) = &schedule.dates.first() {
            match &games.date {
                None => now,
                Some(d) => {
                    if let Ok(p) = NaiveDate::parse_from_str(d, "%Y-%m-%d") {
                        p
                    } else {
                        now
                    }
                }
            }
        } else {
            now
        }
    }
}
