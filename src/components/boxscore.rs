use crate::app::HomeOrAway;
use mlb_api::boxscore::{Player, Team};
use mlb_api::live::LiveResponse;

#[derive(Default)]
pub struct BatterBoxscore {
    order: u8,
    name: String,
    position: String,
    at_bats: u16,
    runs: u16,
    hits: u16,
    rbis: u16,
    walks: u16,
    strike_outs: u16,
    home_runs: u16,
    left_on: u16,
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
            home_runs: player.stats.batting.home_runs.unwrap_or(0),
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
                self.name.split_whitespace().last().unwrap_or("-"),
                self.position
            ),
            self.at_bats.to_string(),
            self.runs.to_string(),
            self.hits.to_string(),
            self.rbis.to_string(),
            self.walks.to_string(),
            self.strike_outs.to_string(),
            self.home_runs.to_string(),
            self.left_on.to_string(),
            self.batting_average.to_string(),
        ]
    }
}

#[derive(Default)]
pub struct TeamBatterBoxscore {
    home_batting: Vec<BatterBoxscore>,
    away_batting: Vec<BatterBoxscore>,
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
                let player = team.players.get(&*format!("ID{}", player_id))?;
                Some(BatterBoxscore::from_data(player, idx as u8 + 1))
            })
            .collect()
    }

    pub fn to_table_row(&self, active: HomeOrAway) -> Vec<Vec<String>> {
        match active {
            HomeOrAway::Home => self.home_batting.iter().map(|p| p.to_vec()).collect(),
            HomeOrAway::Away => self.away_batting.iter().map(|p| p.to_vec()).collect(),
        }
    }
}
