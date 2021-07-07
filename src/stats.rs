use mlb_api::stats::{Split, StatResponse};
use tui::widgets::TableState;

/// Stores the state for rendering the stats.
pub struct StatsState {
    pub state: TableState,
    pub stats: Vec<Stats>,
}

/// Stat information per team.
#[derive(Debug, Default)]
pub struct Stats {
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
        // let s = stats.stats[0];
        stats.stats[0]
            .splits
            .iter()
            .map(|s| Stats::from_pitching_stats(s))
            .collect()
    }

    fn from_pitching_stats(stat: &Split) -> Self {
        Self {
            team_name: stat.team.name.clone(),
            wins: stat.stat.wins,
            losses: stat.stat.losses,
            era: stat.stat.era.clone(),
            games_played: stat.stat.games_played,
            games_started: stat.stat.games_started,
        }
    }

    pub fn to_cells(&self) -> Vec<String> {
        vec![
            self.team_name.clone(),
            self.wins.to_string(),
            self.losses.to_string(),
            self.era.clone(),
            self.games_played.to_string(),
            self.games_started.to_string(),
        ]
    }
}
