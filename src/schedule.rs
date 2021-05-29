use chrono::{DateTime, FixedOffset, Local};
use core::option::Option::{None, Some};
use mlb_api::schedule::{Game, ScheduleResponse};
use std::collections::HashMap;
use tui::widgets::TableState;

pub struct StatefulSchedule {
    pub state: TableState,
    pub schedule: Schedule,
}

impl StatefulSchedule {
    pub fn new(schedule: &ScheduleResponse) -> StatefulSchedule {
        let s = Schedule {
            game_info: Schedule::create_table(schedule),
            game_ids: Schedule::get_game_pks(schedule),
        };
        let mut ss = StatefulSchedule {
            state: TableState::default(),
            schedule: s,
        };
        ss.state.select(Some(0));
        ss
    }

    pub fn update(&mut self, schedule: &ScheduleResponse) {
        self.schedule.game_info = Schedule::create_table(schedule);
        self.schedule.game_ids = Schedule::get_game_pks(schedule);
    }

    pub fn get_selected_game(&self) -> u64 {
        *self
            .schedule
            .game_ids
            .get(
                self.state
                    .selected()
                    .expect("there is always a selected game"),
            )
            .unwrap_or(&0)
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.schedule.game_info.len() - 1 {
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
                    self.schedule.game_info.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub struct Schedule {
    // items: Vec<Vec<&'a str>>, // TODO use &str or String?
    pub game_info: Vec<Vec<String>>,
    pub game_ids: Vec<u64>,
}

impl Schedule {
    /// Create the matchup information to be displayed in the table. The current information that is
    /// extracted from the game data:
    /// away team name, home team name, start time, and game status
    fn create_matchup(game: &Game) -> Vec<String> {
        let home = TEAM_NAME_MAP
            .get(&*game.teams.home.team.name)
            .unwrap()
            .to_string();
        let away = TEAM_NAME_MAP
            .get(&*game.teams.away.team.name)
            .unwrap()
            .to_string();

        // TODO format date for nicer output, this makes return a Vec<&str> impossible. Is this bad?
        let start_time = &game.game_date;
        // TODO let timezone be configurable
        // let est = FixedOffset::west(5 * 60 * 60);
        let pst = FixedOffset::west(8 * 60 * 60);
        let datetime: DateTime<Local> = DateTime::from_utc(
            DateTime::<FixedOffset>::parse_from_rfc3339(start_time)
                .unwrap()
                .naive_utc(),
            pst,
        );
        let formatted = datetime.format("%l:%M %P").to_string();

        let status = match &game.status.detailed_state {
            Some(s) => s.to_string(),
            _ => "-".to_string(),
        };

        vec![away, home, formatted, status]
    }

    /// Generate the scoreboard data to be used to render a table widget.
    fn create_table(schedule: &ScheduleResponse) -> Vec<Vec<String>> {
        let mut todays_games: Vec<Vec<String>> = vec![];
        if let Some(games) = &schedule.dates.get(0) {
            for game in &games.games {
                for g in game {
                    todays_games.push(Schedule::create_matchup(&g));
                }
            }
        }
        todays_games
    }

    fn get_game_pks(schedule: &ScheduleResponse) -> Vec<u64> {
        let mut game_pks: Vec<u64> = vec![];
        if let Some(games) = &schedule.dates.get(0) {
            for game in &games.games {
                for g in game {
                    game_pks.push(g.game_pk);
                }
            }
        }
        game_pks
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
