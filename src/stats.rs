use indexmap::IndexMap;
use mlb_api::stats::{PitchingStat, StatResponse, StatSplit};
use tui::widgets::TableState;

use std::string::ToString;

pub enum StatOption {
    TeamPitching,
    // TeamHitting,
    // PlayerPitching,
    // PlayerHitting,
}

/// Stores the state for rendering the stats.
pub struct StatsState {
    pub state: TableState,
    pub active: StatOption,
    /// stats stores the data in columnar form. The key is the column name and the value contains
    /// the data stored as a vector. `IndexMap` is used to store the data in inserted order, which
    /// enables deterministic access of the data (for transforming to row oriented).
    pub stats: IndexMap<String, TableEntry>,
}

#[derive(Clone)]
pub struct TableEntry {
    pub description: String,
    pub active: bool,
    pub rows: Vec<String>,
}

impl Default for StatsState {
    fn default() -> Self {
        let mut ss = StatsState {
            state: TableState::default(),
            active: StatOption::TeamPitching,
            stats: IndexMap::new(),
        };
        ss.state.select(Some(0));
        ss
    }
}

impl StatsState {
    pub fn update(&mut self, stats: &StatResponse) {
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

    pub fn generate_table(&self) -> (Vec<String>, Vec<Vec<String>>) {
        if self.stats.is_empty() {
            return (vec![], vec![vec![]]);
        }

        let len = match self.active {
            StatOption::TeamPitching => self.stats.get("Team").unwrap().rows.len(),
            // StatOption::TeamHitting => self.stats.get("Team").unwrap().rows.len(),
            // StatOption::PlayerPitching => self.stats.get("Player").unwrap().rows.len(),
        };
        let mut rows = vec![Vec::new(); len];
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

    fn table_helper(&mut self, key: &str, description: &str, active: bool, value: &str) {
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

    fn from_pitching_stats(&mut self, name: String, stat: &PitchingStat) {
        self.table_helper("Team", "", true, &name);
        self.table_helper("W", "wins", true, &stat.wins.to_string());
        self.table_helper("L", "losses", true, &stat.losses.to_string());
        self.table_helper("ERA", "earned run average", true, &stat.era);
        self.table_helper("G", "game played", true, &stat.games_played.to_string());
        self.table_helper("GS", "games started", true, &stat.games_started.to_string());
    }

    // TODO enable table selection
    // pub fn next(&mut self) {
    //     let i = match self.state.selected() {
    //         Some(i) => {
    //             if i >= self.stats.len() - 1 {
    //                 0
    //             } else {
    //                 i + 1
    //             }
    //         }
    //         None => 0,
    //     };
    //     self.state.select(Some(i));
    // }

    // pub fn previous(&mut self) {
    //     let i = match self.state.selected() {
    //         Some(i) => {
    //             if i == 0 {
    //                 self.stats.len() - 1
    //             } else {
    //                 i - 1
    //             }
    //         }
    //         None => 0,
    //     };
    //     self.state.select(Some(i));
    // }
}
