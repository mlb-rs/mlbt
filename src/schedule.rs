use mlb_api::schedule::{Game, ScheduleResponse};

use crate::app::HomeOrAway;

use chrono::{DateTime, Datelike, NaiveDate, ParseError, Utc};
use chrono_tz::America::Los_Angeles;
use core::option::Option::{None, Some};
use lazy_static::lazy_static;
use tui::widgets::TableState;

use std::cmp::Ordering;
use std::collections::HashMap;

/// ScheduleState is used to render the schedule as a `tui-rs` table.
pub struct ScheduleState {
    pub state: TableState,
    pub schedule: Vec<ScheduleRow>,
    pub date: NaiveDate,
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
    pub fn from_schedule(schedule: &ScheduleResponse) -> Self {
        let mut ss = ScheduleState {
            state: TableState::default(),
            schedule: ScheduleRow::create_table(schedule),
            date: ScheduleRow::get_date_from_schedule(schedule),
        };
        ss.state.select(Some(0));
        ss
    }

    /// Update the data from the API. It is assumed that the date is already updated, aka don't use
    /// a random date without first setting the `date` field. Use `set_date_from_input` for this.
    pub fn update(&mut self, schedule: &ScheduleResponse) {
        self.schedule = ScheduleRow::create_table(schedule);
    }

    /// Set the date from the input string from the date picker.
    pub fn set_date_from_input(&mut self, date: String) -> Result<(), ParseError> {
        self.date = match date.as_str() {
            "today" => {
                let today = Utc::now().naive_local();
                NaiveDate::from_ymd(today.year(), today.month(), today.day())
            }
            _ => NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d")?,
        };
        self.state.select(Some(0));
        Ok(())
    }

    /// Return the `game_id` of the row that is selected.
    pub fn get_selected_game(&self) -> u64 {
        match self.schedule.get(
            self.state
                .selected()
                .expect("there is always a selected game"),
        ) {
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
        let home = match self.home_score {
            Some(h) => h,
            None => return None,
        };
        let away = match self.away_score {
            Some(a) => a,
            None => return None,
        };

        match home.cmp(&away) {
            Ordering::Greater => Some(HomeOrAway::Home),
            Ordering::Less => Some(HomeOrAway::Away),
            Ordering::Equal => None,
        }
    }

    /// Create the matchup information to be displayed in the table. The current information that is
    /// extracted from the game data:
    /// away team name and score, home team name and score, start time, and game status
    fn create_matchup(game: &Game) -> Self {
        let home_team = TEAM_NAME_MAP
            .get(&*game.teams.home.team.name)
            .unwrap()
            .to_string();

        let away_team = TEAM_NAME_MAP
            .get(&*game.teams.away.team.name)
            .unwrap()
            .to_string();

        // TODO let timezone be configurable
        let datetime = DateTime::parse_from_rfc3339(&game.game_date)
            .unwrap()
            .with_timezone(&Los_Angeles);
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
    fn create_table(schedule: &ScheduleResponse) -> Vec<Self> {
        let mut todays_games: Vec<ScheduleRow> = Vec::new();
        if let Some(games) = &schedule.dates.get(0) {
            for game in &games.games {
                for g in game {
                    todays_games.push(ScheduleRow::create_matchup(&g));
                }
            }
        }
        todays_games
    }

    /// The date is stored in schedule -> dates -> date.
    fn get_date_from_schedule(schedule: &ScheduleResponse) -> NaiveDate {
        let now = Utc::now().naive_local();
        let now = NaiveDate::from_ymd(now.year(), now.month(), now.day());
        return if let Some(games) = &schedule.dates.get(0) {
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
        };
    }
}

lazy_static! {
    /// This maps the full name of a team to its short name. The short name is used in the boxscore.
    /// The team names are taken from the `teams` endpoint.
    static ref TEAM_NAME_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Oakland Athletics", "Athletics");
        m.insert("Pittsburgh Pirates", "Pirates");
        m.insert("San Diego Padres", "Padres");
        m.insert("Seattle Mariners", "Mariners");
        m.insert("San Francisco Giants", "Giants");
        m.insert("St. Louis Cardinals", "Cardinals");
        m.insert("Tampa Bay Rays", "Rays");
        m.insert("Texas Rangers", "Rangers");
        m.insert("Toronto Blue Jays", "Blue Jays");
        m.insert("Minnesota Twins", "Twins");
        m.insert("Philadelphia Phillies", "Phillies");
        m.insert("Atlanta Braves", "Braves");
        m.insert("Chicago White Sox", "White Sox");
        m.insert("Miami Marlins", "Marlins");
        m.insert("Florida Marlins", "Marlins");
        m.insert("New York Yankees", "Yankees");
        m.insert("Milwaukee Brewers", "Brewers");
        m.insert("Los Angeles Angels", "Angels");
        m.insert("Arizona Diamondbacks", "D-backs");
        m.insert("Baltimore Orioles", "Orioles");
        m.insert("Boston Red Sox", "Red Sox");
        m.insert("Chicago Cubs", "Cubs");
        m.insert("Cincinnati Reds", "Reds");
        m.insert("Cleveland Indians", "Indians");
        m.insert("Colorado Rockies", "Rockies");
        m.insert("Detroit Tigers", "Tigers");
        m.insert("Houston Astros", "Astros");
        m.insert("Kansas City Royals", "Royals");
        m.insert("Los Angeles Dodgers", "Dodgers");
        m.insert("Washington Nationals", "Nationals");
        m.insert("New York Mets", "Mets");
        m
    };
}
