use crate::components::standings::Team;
use crate::components::util::{era_color, OptionDisplayExt};
use mlbt_api::schedule::TeamInfo;
use tui::prelude::Stylize;
use tui::style::Color;
use tui::widgets::Cell;

#[derive(Debug, Clone)]
pub struct ProbablePitcher {
    pub name: String,
    pub strike_outs: Option<u16>,
    pub base_on_balls: Option<u16>,
    pub era: Option<String>,
    pub innings_pitched: Option<String>,
    pub wins: Option<u8>,
    pub losses: Option<u8>,
}

pub struct ProbablePitcherMatchup<'a> {
    pub home_pitcher: &'a ProbablePitcher,
    pub home_team: Team,
    pub away_pitcher: &'a ProbablePitcher,
    pub away_team: Team,
}

impl Default for ProbablePitcher {
    fn default() -> Self {
        ProbablePitcher {
            name: "TBD".to_string(),
            strike_outs: None,
            base_on_balls: None,
            era: None,
            innings_pitched: None,
            wins: None,
            losses: None,
        }
    }
}

impl ProbablePitcher {
    pub fn from_team(team: &TeamInfo) -> Option<Self> {
        let pitcher = team.probable_pitcher.as_ref()?;
        let stats = pitcher.stats.iter().find_map(|entry| {
            // check if this entry belongs to the "pitching" group and is season stats
            let is_pitching = entry
                .group
                .as_ref()
                .is_some_and(|g| g.display_name == "pitching")
                && entry
                    .stat_type
                    .as_ref()
                    .is_some_and(|t| t.display_name == "statsSingleSeason");

            if is_pitching {
                entry.stats.as_ref()
            } else {
                None
            }
        });
        Some(Self {
            name: pitcher.full_name.clone(),
            strike_outs: stats.and_then(|s| s.strike_outs),
            base_on_balls: stats.and_then(|s| s.base_on_balls),
            era: stats.and_then(|s| s.era.clone()),
            innings_pitched: stats.and_then(|s| s.innings_pitched.clone()),
            wins: stats.and_then(|s| s.wins),
            losses: stats.and_then(|s| s.losses),
        })
    }

    pub fn to_row_cells(&self, team_name: &str) -> Vec<Cell<'static>> {
        vec![
            Cell::from(team_name.to_string()),
            Cell::from(self.name.clone()),
            Cell::from(self.wins.display_or("-")),
            Cell::from(self.losses.display_or("-")),
            Cell::from(self.era.display_or("-"))
                .fg(self.era.as_deref().and_then(era_color).unwrap_or(Color::White)),
            Cell::from(self.innings_pitched.display_or("-")),
            Cell::from(self.strike_outs.display_or("-")),
            Cell::from(self.base_on_balls.display_or("-")),
        ]
    }
}
