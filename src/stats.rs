use mlb_api::stats::{PitchingStat, StatResponse};
use tui::widgets::TableState;

/// Stores the state for rendering the stats.
pub struct StatsState {
    pub state: TableState,
    pub stats: Vec<Stats>,
}

pub enum Stats {
    Pitching(PStats),
}

/// Stat information per team.
#[derive(Debug, Default)]
pub struct PStats {
    pub team_name: String,
    pub wins: u16,
    pub losses: u16,
    pub era: String,
    pub games_played: u16,
    pub games_started: u16,
}

impl Default for StatsState {
    fn default() -> Self {
        let mut ss = StatsState {
            state: TableState::default(),
            stats: vec![],
        };
        ss.state.select(Some(0));
        ss
    }
}

impl StatsState {
    pub fn update(&mut self, stats: &StatResponse) {
        self.stats = Stats::create_table(stats);
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.stats.len() - 1 {
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
                    self.stats.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl Stats {
    /// Generate the stats data to be used to render a table widget.
    fn create_table(stats: &StatResponse) -> Vec<Stats> {
        // TODO loop over all stats
        stats.stats[0]
            .splits
            .iter()
            .map(|s| {
                let name = s.team.name.clone();
                match &s.stat {
                    mlb_api::stats::StatSplit::Pitching(p) => Stats::from_pitching_stats(name, p),
                    mlb_api::stats::StatSplit::Hitting(_) => todo!(),
                }
            })
            .collect()
    }

    // TODO add hitting stats

    fn from_pitching_stats(name: String, stat: &PitchingStat) -> Stats {
        let p = PStats {
            team_name: name,
            wins: stat.wins,
            losses: stat.losses,
            era: stat.era.clone(),
            games_played: stat.games_played,
            games_started: stat.games_started,
        };
        Stats::Pitching(p)
    }

    pub fn to_cells(&self) -> Vec<String> {
        match self {
            Stats::Pitching(s) => {
                vec![
                    s.team_name.clone(),
                    s.wins.to_string(),
                    s.losses.to_string(),
                    s.era.clone(),
                    s.games_played.to_string(),
                    s.games_started.to_string(),
                ]
            }
        }
    }
}
