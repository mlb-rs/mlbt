use mlbt_api::schedule::Game;

#[derive(Debug, Clone)]
pub struct DecisionPitcher {
    pub name: String,
    pub era: Option<String>,
    pub wins: Option<u8>,
    pub losses: Option<u8>,
    pub saves: Option<u8>,
}

/// Before a game is Final, there are no decision pitchers.
#[derive(Debug, Clone)]
pub struct GameDecisionPitchers {
    pub winning_pitcher: DecisionPitcher,
    pub losing_pitcher: DecisionPitcher,
    pub save_pitcher: Option<DecisionPitcher>,
}

impl DecisionPitcher {
    /// Surname for compact display. Skips trailing generational suffixes so
    /// "Vladimir Guerrero Jr." returns "Guerrero" instead of "Jr.".
    pub fn last_name(&self) -> &str {
        let mut parts = self.name.rsplitn(3, ' ');
        let tail = parts.next().unwrap_or(&self.name);
        if matches!(tail, "Jr." | "Sr." | "II" | "III" | "IV") {
            parts.next().unwrap_or(tail)
        } else {
            tail
        }
    }
}

impl From<&mlbt_api::schedule::DecisionPitcher> for DecisionPitcher {
    fn from(pitcher: &mlbt_api::schedule::DecisionPitcher) -> Self {
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
        Self {
            name: pitcher.full_name.clone(),
            era: stats.and_then(|s| s.era.clone()),
            wins: stats.and_then(|s| s.wins),
            losses: stats.and_then(|s| s.losses),
            saves: stats.and_then(|s| s.saves),
        }
    }
}

impl GameDecisionPitchers {
    pub fn from_game(game: &Game) -> Option<Self> {
        let decisions = game.decisions.as_ref()?;
        Some(Self {
            winning_pitcher: DecisionPitcher::from(&decisions.winner),
            losing_pitcher: DecisionPitcher::from(&decisions.loser),
            save_pitcher: decisions.save.as_ref().map(DecisionPitcher::from),
        })
    }

    /// Number of pitcher lines to render: W and L always, plus S when present.
    pub fn count(&self) -> u16 {
        if self.save_pitcher.is_some() { 3 } else { 2 }
    }
}
