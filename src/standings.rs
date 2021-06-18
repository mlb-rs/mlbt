use core::option::Option::{None, Some};
use lazy_static::lazy_static;
use mlb_api::standings::StandingsResponse;
use std::collections::HashMap;
use tui::widgets::TableState;

pub struct StandingsState {
    pub state: TableState,
    pub standings: Vec<Standing>,
}

impl StandingsState {
    pub fn from_standings(standings: &StandingsResponse) -> Self {
        let mut ss = StandingsState {
            state: TableState::default(),
            standings: Standing::create_table(standings),
        };
        ss.state.select(Some(0));
        ss
    }

    pub fn update(&mut self, standings: &StandingsResponse) {
        self.standings = Standing::create_table(standings);
    }

    pub fn get_selected(&self) -> u16 {
        if let Some(s) = self.standings.get(
            self.state
                .selected()
                .expect("there is always a selected standing"),
        ) {
            s.team_id
        } else {
            0
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.standings.len() - 1 {
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
                    self.standings.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

#[derive(Debug, Default)]
pub struct Standing {
    pub team_name: String,
    pub team_id: u16,
    pub wins: u8,
    pub losses: u8,
    pub winning_percentage: String,
    pub games_back: String,
    pub wild_card_games_back: String,
    pub streak: String,
}

impl Standing {
    /// Generate the standings data to be used to render a table widget.
    fn create_table(standings: &StandingsResponse) -> Vec<Standing> {
        let mut asdf: Vec<Standing> = vec![];
        for record in &standings.records {
            // TODO league and division
            for team in &record.team_records {
                let s = Standing {
                    team_name: team.team.name.clone(),
                    team_id: team.team.id,
                    wins: team.wins,
                    losses: team.losses,
                    winning_percentage: team.winning_percentage.clone(),
                    games_back: team.games_back.clone(),
                    wild_card_games_back: team.wild_card_games_back.clone(),
                    streak: team.streak.streak_code.clone(),
                };
                asdf.push(s);
            }
        }
        asdf
    }

    pub fn to_cells(&self) -> Vec<String> {
        vec![
            self.team_name.clone(),
            self.wins.to_string(),
            self.losses.to_string(),
            self.winning_percentage.clone(),
            self.games_back.clone(),
            self.wild_card_games_back.clone(),
            self.streak.clone(),
        ]
    }
}

lazy_static! {
    /// This maps the `teamId` to the `shortName` for each division and league.
    /// The team names are taken from the `divisions` endpoint.
    static ref DIVISION_MAP: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
        m.insert(103, "American League");
        m.insert(104, "National League");
        m.insert(200, "AL West");
        m.insert(201, "AL East");
        m.insert(202, "AL Central");
        m.insert(203, "NL West");
        m.insert(204, "NL East");
        m.insert(205, "NL Central");
        m
    };
}
