use crate::app::App;
use chrono::{DateTime, FixedOffset, Local};
use core::option::Option::{None, Some};
use mlb_api::schedule::{Game, ScheduleResponse};
use mlb_api::MLBApi;
use std::collections::HashMap;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

pub struct StatefulSchedule {
    pub state: TableState,
    // items: Vec<Vec<&'a str>>, // TODO use &str or String?
    pub items: Vec<Vec<String>>,
    pub game_ids: Vec<u64>,
}

impl StatefulSchedule {
    pub fn new(schedule: &ScheduleResponse) -> StatefulSchedule {
        StatefulSchedule {
            state: TableState::default(),
            items: create_table(schedule),
            game_ids: get_game_pks(schedule),
        }
    }

    pub fn update_schedule(&mut self, api: &MLBApi) {
        let schedule = api.get_todays_schedule().unwrap(); // TODO add error handling
        self.items = create_table(&schedule);
        self.game_ids = get_game_pks(&schedule);
    }

    pub fn get_selected_game(&self) -> u64 {
        *self
            .game_ids
            .get(
                self.state
                    .selected()
                    .expect("there is always a selected game"),
            )
            .expect("a game always has an id")
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
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
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub fn render_schedule<B>(f: &mut Frame<B>, rect: Rect, app: &mut App)
where
    B: Backend,
{
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::White);
    let header_cells = ["away", "home", "time [PST]", "status"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));

    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

    let rows = app
        .schedule
        .items
        .iter()
        .map(|r| Row::new(r.clone()).height(1).bottom_margin(1));

    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("scoreboard"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            // TODO review these on different width terminals
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(45),
        ]);

    f.render_stateful_widget(t, rect, &mut app.schedule.state);
}

pub fn create_matchup(game: &Game) -> Vec<String> {
    let h_short = TEAM_NAME_MAP
        .get(&*game.teams.home.team.name)
        .unwrap()
        .to_string();
    let a_short = TEAM_NAME_MAP
        .get(&*game.teams.away.team.name)
        .unwrap()
        .to_string();

    // TODO format date for nicer output, this makes return a Vec<&str> impossible. Is this bad?
    let start_time = &game.game_date;
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
        _ => "unknown".to_string(),
    };

    vec![a_short, h_short, formatted, status]
}

/// Generate the scoreboard data to be used to render a table widget.
pub fn create_table(schedule: &ScheduleResponse) -> Vec<Vec<String>> {
    // TODO expecting only to grab one day of schedule at a time, but this is kind of brittle
    let today = &schedule.dates[0];
    let mut todays_games: Vec<Vec<String>> = vec![];
    for game in &today.games {
        for g in game {
            todays_games.push(create_matchup(g));
        }
    }
    todays_games
}

pub fn get_game_pks(schedule: &ScheduleResponse) -> Vec<u64> {
    // TODO expecting only to grab one day of schedule at a time, but this is kind of brittle
    let today = &schedule.dates[0];
    let mut game_pks: Vec<u64> = vec![];
    for game in &today.games {
        for g in game {
            game_pks.push(g.game_pk);
        }
    }
    game_pks
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
