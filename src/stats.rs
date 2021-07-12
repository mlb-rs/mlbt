use indexmap::IndexMap;
use mlb_api::stats::{PitchingStat, StatResponse, StatSplit};
use tui::widgets::TableState;

use std::string::ToString;

/// The width of the first column, which is a longer item like team name.
pub const STATS_FIRST_COL_WIDTH: u16 = 25;
/// The width of normal columns.
pub const STATS_DEFAULT_COL_WIDTH: u16 = 6;

pub enum StatOption {
    TeamPitching,
    // TeamHitting,
    // PlayerPitching,
    // PlayerHitting,
}

/// Stores the state for rendering the stats.
pub struct StatsState {
    pub state: TableState,
    pub stat_type: StatOption,
    /// Stores the data in columnar form. The key is the column name and the value contains the
    /// data stored as a vector. `IndexMap` is used to store the data in inserted order, which
    /// enables deterministic access of the data (for transforming to row oriented).
    pub stats: IndexMap<String, TableEntry>,
    /// Whether to display the side bar for toggling stats
    pub stats_options: bool,
}

/// The information for a stat, including all the data values.
pub struct TableEntry {
    /// Longer description of the stat to be displayed in the options toggle pane.
    pub description: String,
    /// Whether to display the column or not.
    pub active: bool,
    /// The data values. Note they are stored as strings to allow for creation of a tui-rs Cell.
    pub rows: Vec<String>,
}

impl Default for StatsState {
    fn default() -> Self {
        let mut ss = StatsState {
            state: TableState::default(),
            stat_type: StatOption::TeamPitching,
            stats: IndexMap::new(),
            stats_options: true,
        };
        ss.state.select(Some(0));
        ss
    }
}

impl StatsState {
    pub fn update(&mut self, stats: &StatResponse) {
        self.stats.clear();
        self.create_table(stats);
    }

    fn create_table(&mut self, stats: &StatResponse) {
        for stat in &stats.stats {
            for split in &stat.splits {
                let name = split.team.name.clone();
                match &split.stat {
                    // TODO do I need to differentiate between team/player stats here?
                    StatSplit::Pitching(p) => self.from_pitching_stats(name, p),
                    StatSplit::Hitting(_) => todo!(),
                };
            }
        }
    }

    /// Create the header and the table rows from the table map. Basically transforms from columnar
    /// to row based, filtering on whether the data is active.
    pub fn generate_table(&self) -> (Vec<String>, Vec<Vec<String>>) {
        if self.stats.is_empty() {
            return (vec![], vec![vec![]]);
        }

        let len = match self.stat_type {
            StatOption::TeamPitching => self.stats.get("Team").unwrap().rows.len(),
            // StatOption::TeamHitting => self.stats.get("Team").unwrap().rows.len(),
            // StatOption::PlayerPitching => self.stats.get("Player").unwrap().rows.len(),
        };
        let mut rows = vec![Vec::with_capacity(self.stats.len()); len];
        let mut header = Vec::with_capacity(self.stats.len());

        // access the data in stored order because of `IndexMap`
        for (key, col) in &self.stats {
            if col.active {
                header.push(key.clone());
                for (idx, val) in col.rows.iter().enumerate() {
                    rows[idx].push(val.clone());
                }
            }
        }
        (header, rows)
    }

    /// Insert stats into the table map. If the key isn't present a new column is created, otherwise
    /// the data is simply added.
    fn table_helper<T>(&mut self, key: &str, description: &str, active: bool, value: T)
    where
        T: ToString,
    {
        if !self.stats.contains_key(key) {
            self.stats.insert(
                key.to_string(),
                TableEntry {
                    description: description.to_string(),
                    active,
                    rows: vec![value.to_string()],
                },
            );
        } else {
            self.stats
                .get_mut(key)
                .unwrap()
                .rows
                .push(value.to_string());
        }
    }

    /// Create the pitching stats table. Note that the order of the calls to `table_helper` is the
    /// order in which the stats will be displayed from left to right.
    fn from_pitching_stats(&mut self, name: String, stat: &PitchingStat) {
        self.table_helper("Team", "", true, name);
        self.table_helper("W", "wins", true, stat.wins);
        self.table_helper("L", "losses", true, stat.losses);
        self.table_helper("ERA", "earned run average", true, &stat.era);
        self.table_helper("G", "game played", true, stat.games_played);
        self.table_helper("GS", "games started", true, stat.games_started);
        self.table_helper("CG", "complete games", true, stat.complete_games);
        self.table_helper("SHO", "shutouts", false, stat.shutouts);
        self.table_helper("SV", "saves", true, stat.saves);
        self.table_helper("SVO", "save opportunities", true, stat.save_opportunities);
        self.table_helper("IP", "innings pitched", true, &stat.innings_pitched);
        self.table_helper("H", "hits", true, stat.hits);
        self.table_helper("R", "runs", true, stat.runs);
        self.table_helper("ER", "earned runs", true, stat.earned_runs);
        self.table_helper("HR", "home runs", true, stat.home_runs);
        self.table_helper("HB", "hit batsmen", false, stat.hit_batsmen);
        self.table_helper("BB", "walks", true, stat.base_on_balls);
        self.table_helper("SO", "strike outs", true, stat.strike_outs);
    }

    /// Deactivate columns that would overflow the available width.
    pub fn trim_columns(&mut self, available_width: u16) {
        // Get the indices of active columns
        let mut active: Vec<usize> = self
            .stats
            .values()
            .enumerate()
            .filter(|(_, v)| v.active)
            .map(|(i, _)| i)
            .collect();

        // calculate total width of active columns
        let mut column_width = (active.len() as u16 * STATS_DEFAULT_COL_WIDTH)
            + (STATS_FIRST_COL_WIDTH - STATS_DEFAULT_COL_WIDTH) // add remainder of first column
            - 2; // 2 for left/right borders

        // start deactivating columns as needed, from left to right
        while column_width >= available_width && !active.is_empty() {
            let key = active.pop().unwrap();
            if let Some((_, v)) = self.stats.get_index_mut(key) {
                v.active = false;
                column_width -= STATS_DEFAULT_COL_WIDTH;
            }
        }
    }

    /// Toggle the visibility of the stat column that is selected.
    pub fn toggle_stat(&mut self) {
        if let Some((_, v)) = self.stats.get_index_mut(
            self.state
                .selected()
                .expect("there is always a selected stat"),
        ) {
            v.active = !v.active;
        }
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
