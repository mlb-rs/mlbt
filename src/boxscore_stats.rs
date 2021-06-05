use crate::app::BoxscoreTab;
use mlb_api::boxscore::{Player, Team};
use mlb_api::live::LiveResponse;

#[derive(Default)]
pub struct BatterBoxscore {
    order: u8,
    name: String,
    position: String,
    at_bats: u16,
    runs: u8,
    hits: u8,
    rbis: u8,
    walks: u8,
    strike_outs: u8,
    left_on: u8,
    batting_average: String,
}

impl BatterBoxscore {
    pub fn from_data(player: &Player, order: u8) -> Self {
        BatterBoxscore {
            order,
            name: player.person.full_name.to_string(),
            position: player.position.abbreviation.to_string(),
            at_bats: player.stats.batting.at_bats.unwrap_or(0),
            runs: player.stats.batting.runs.unwrap_or(0),
            hits: player.stats.batting.hits.unwrap_or(0),
            rbis: player.stats.batting.rbi.unwrap_or(0),
            walks: player.stats.batting.base_on_balls.unwrap_or(0),
            strike_outs: player.stats.batting.strike_outs.unwrap_or(0),
            left_on: player.stats.batting.left_on_base.unwrap_or(0),
            batting_average: player
                .season_stats
                .batting
                .avg
                .clone()
                .unwrap_or_else(|| "---".to_string()),
        }
    }
    pub fn to_vec(&self) -> Vec<String> {
        // let header = vec!["player", "ab", "r", "h", "rbi", "bb", "so", "lob", "avg"];
        vec![
            format!(
                "{} {} {}",
                self.order,
                self.name
                    .split_whitespace()
                    .last()
                    .unwrap_or(&"-".to_string()),
                self.position
            ),
            self.at_bats.to_string(),
            self.runs.to_string(),
            self.hits.to_string(),
            self.rbis.to_string(),
            self.walks.to_string(),
            self.strike_outs.to_string(),
            self.left_on.to_string(),
            self.batting_average.to_string(),
        ]
    }
}

pub struct TeamBatterBoxscore {
    home_batting: Vec<BatterBoxscore>,
    away_batting: Vec<BatterBoxscore>,
}

impl Default for TeamBatterBoxscore {
    fn default() -> Self {
        TeamBatterBoxscore {
            home_batting: vec![],
            away_batting: vec![],
        }
    }
}

impl TeamBatterBoxscore {
    pub fn from_live_data(live_game: &LiveResponse) -> Self {
        let (home, away) = match &live_game.live_data.boxscore.teams {
            Some(t) => (&t.home, &t.away),
            None => return TeamBatterBoxscore::default(),
        };
        TeamBatterBoxscore {
            home_batting: TeamBatterBoxscore::transform(home),
            away_batting: TeamBatterBoxscore::transform(away),
        }
    }

    fn transform(team: &Team) -> Vec<BatterBoxscore> {
        team.batting_order
            .iter()
            .enumerate()
            .filter_map(|(idx, player_id)| {
                let player = match team.players.get(&*format!("ID{}", player_id)) {
                    Some(p) => p,
                    None => return None,
                };
                Some(BatterBoxscore::from_data(player, idx as u8 + 1))
            })
            .collect()
    }

    pub fn to_table_row(&self, active: BoxscoreTab) -> Vec<Vec<String>> {
        match active {
            BoxscoreTab::Home => self.home_batting.iter().map(|p| p.to_vec()).collect(),
            BoxscoreTab::Away => self.away_batting.iter().map(|p| p.to_vec()).collect(),
        }
    }
}
