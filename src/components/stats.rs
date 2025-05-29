use crate::components::date_selector::DateSelector;
use chrono::NaiveDate;
use indexmap::IndexMap;
use mlb_api::client::StatGroup;
use mlb_api::stats::{HittingStat, PitchingStat, StatResponse, StatSplit};
use std::cmp::Ordering;
use std::string::ToString;
use tui::widgets::TableState;

/// The width of the first column, which is a longer item like team name.
pub const STATS_FIRST_COL_WIDTH: u16 = 25;
/// The width of normal columns.
pub const STATS_DEFAULT_COL_WIDTH: u16 = 6;
const PLAYER_COLUMN_NAME: &str = "Player";
const TEAM_COLUMN_NAME: &str = "Team";

/// Stores whether a team/player and pitching/hitting stat should be viewed.
#[derive(Clone, Debug)]
pub struct StatType {
    pub group: StatGroup,
    pub team_player: TeamOrPlayer,
}

#[derive(Clone, Debug, Default)]
pub enum TeamOrPlayer {
    #[default]
    Team,
    Player,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Order {
    #[default]
    Ascending,
    Descending,
}

impl std::ops::Not for Order {
    type Output = Self;

    // Useful for toggling between order directions
    fn not(self) -> Self::Output {
        match self {
            Order::Ascending => Order::Descending,
            Order::Descending => Order::Ascending,
        }
    }
}

impl Order {
    /// Returns an arrow character representing the direction.
    pub fn arrow_symbol(&self) -> &'static str {
        match self {
            Order::Ascending => "^",
            Order::Descending => "v",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Sort {
    pub column_name: Option<String>,
    pub order: Order,
}

impl Default for Sort {
    fn default() -> Self {
        Sort {
            column_name: None,
            order: Order::Ascending,
        }
    }
}

/// Stores the state for rendering the stats.
pub struct StatsState {
    pub state: TableState,
    /// The stat type combination of team/player and pitching/hitting.
    pub stat_type: StatType,
    /// Stores the data in columnar form. The key is the column name and the value contains the
    /// data stored as a vector. `IndexMap` is used to store the data in inserted order, which
    /// enables deterministic access of the data (for transforming to row oriented).
    pub stats: IndexMap<String, TableEntry>,
    /// The column and direction used for sorting the stats
    pub sorting: Sort,
    /// Whether to display the side bar for toggling stats
    pub show_options: bool,
    /// Date selector for viewing stats on a specific date.
    pub date_selector: DateSelector,
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
            stat_type: StatType {
                group: StatGroup::Pitching,
                team_player: TeamOrPlayer::Team,
            },
            stats: IndexMap::new(),
            sorting: Sort::default(),
            show_options: true,
            date_selector: DateSelector::default(),
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

    /// Set the date from the validated input string from the date picker.
    pub fn set_date_from_valid_input(&mut self, date: NaiveDate) {
        self.date_selector.set_date_from_valid_input(date);
        self.state.select(Some(0));
    }

    /// Set the date using Left/Right arrow keys to move a single day at a time.
    pub fn set_date_with_arrows(&mut self, forward: bool) -> NaiveDate {
        self.date_selector.set_date_with_arrows(forward)
    }

    fn create_table(&mut self, stats: &StatResponse) {
        for stat in &stats.stats {
            for split in &stat.splits {
                // if the `player` field exists, then its a Player stat
                let name = match &split.player {
                    Some(p) => p.full_name.clone(),
                    None => split.team.name.clone(),
                };
                match &split.stat {
                    StatSplit::Pitching(s) => self.load_pitching_stats(name, s),
                    StatSplit::Hitting(s) => self.load_hitting_stats(name, s),
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

        // get the number of rows, which might be zero while data is loading
        let len = match self.stats.first() {
            Some((_, v)) => v.rows.len(),
            None => 0,
        };
        if len == 0 {
            return (vec![], vec![vec![]]);
        }
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

        self.sort_table(&mut rows);

        (header, rows)
    }

    /// Insert stats into the table map. If the key isn't present a new column is created, otherwise
    /// the data is simply added.
    fn table_helper<T>(&mut self, key: &str, description: &str, active: bool, value: T)
    where
        T: ToString,
    {
        self.stats
            .entry(key.to_string())
            .and_modify(|table_entry| table_entry.rows.push(value.to_string()))
            .or_insert(TableEntry {
                description: description.to_string(),
                active,
                rows: vec![value.to_string()],
            });
    }

    /// Create the pitching stats table. Note that the order of the calls to `table_helper` is the
    /// order in which the stats will be displayed from left to right.
    fn load_pitching_stats(&mut self, name: String, stat: &PitchingStat) {
        let col_name = match self.stat_type.team_player {
            TeamOrPlayer::Team => TEAM_COLUMN_NAME,
            TeamOrPlayer::Player => PLAYER_COLUMN_NAME,
        };
        self.table_helper(col_name, "", true, name);
        self.table_helper("W", "wins", true, stat.wins);
        self.table_helper("L", "losses", true, stat.losses);
        self.table_helper("ERA", "earned run average", true, &stat.era);
        self.table_helper("G", "games played", true, stat.games_played);
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

    /// Create the hitting stats table. Note that the order of the calls to `table_helper` is the
    /// order in which the stats will be displayed from left to right.
    fn load_hitting_stats(&mut self, name: String, stat: &HittingStat) {
        let col_name = match self.stat_type.team_player {
            TeamOrPlayer::Team => "Team",
            TeamOrPlayer::Player => "Player",
        };
        self.table_helper(col_name, "", true, name);
        self.table_helper("G", "games played", true, stat.games_played);
        self.table_helper("AB", "at bats", true, stat.at_bats);
        self.table_helper("AVG", "batting avg", true, &stat.avg);
        self.table_helper("OBP", "on-base percent", true, &stat.obp);
        self.table_helper("SLG", "slugging percent", true, &stat.slg);
        self.table_helper("OPS", "on-base + slug", true, &stat.ops);
        self.table_helper("R", "runs", true, stat.runs);
        self.table_helper("H", "hits", true, stat.hits);
        self.table_helper("2B", "doubles", true, stat.doubles);
        self.table_helper("3B", "triples", true, stat.triples);
        self.table_helper("HR", "home runs", true, stat.home_runs);
        self.table_helper("RBI", "runs batted in", true, stat.rbi);
        self.table_helper("BB", "walks", true, stat.base_on_balls);
        self.table_helper("SO", "strike outs", true, stat.strike_outs);
        self.table_helper("SB", "stolen bases", true, stat.stolen_bases);
        self.table_helper("CS", "caught stealing", true, stat.caught_stealing);
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
        let toggled_column_index = self.state.selected().unwrap_or_default();
        let sort_column_index = self.get_sort_column_index();

        if let Some((_, v)) = self.stats.get_index_mut(toggled_column_index) {
            v.active = !v.active;
            // if the column is the sort column and it's toggled off, reset the sorting
            if sort_column_index.is_some_and(|idx| idx == toggled_column_index) && !v.active {
                self.sorting.column_name = None;
            }
        }
    }

    /// Get the index of the sort column while taking into account if columns to the left of it are
    /// active.
    fn get_sort_column_index(&self) -> Option<usize> {
        let sort_column = self.sorting.column_name.as_ref()?;

        let mut active_idx = 0;
        for (column_name, entry) in self.stats.iter() {
            if column_name == sort_column {
                return Some(active_idx);
            }
            if entry.active {
                active_idx += 1;
            }
        }
        None
    }

    /// Sort the table by this stat.
    pub fn store_sort_column(&mut self) {
        let Some(selected_index) = self.state.selected() else {
            return;
        };

        // if the stat isn't already active don't sort
        if let Some((column_name, entry)) = self.stats.get_index(selected_index) {
            if !entry.active {
                return;
            }
            self.sorting = Sort {
                column_name: Some(column_name.clone()),
                order: !self.sorting.order,
            };
        }
    }

    /// Sort the rows by the selected stat.
    fn sort_table(&self, rows: &mut [Vec<String>]) {
        let sort_column_index = self.get_sort_column_index();
        let sort_column_name = self.sorting.column_name.as_ref();

        if let (Some(sort_column_index), Some(sort_column)) = (sort_column_index, sort_column_name)
        {
            rows.sort_by(|a, b| {
                let a = a.get(sort_column_index);
                let b = b.get(sort_column_index);
                if let (Some(a), Some(b)) = (a, b) {
                    // if the column is a name don't try to convert to float
                    if sort_column == TEAM_COLUMN_NAME || sort_column == PLAYER_COLUMN_NAME {
                        match self.sorting.order {
                            Order::Ascending => a.cmp(b),
                            Order::Descending => b.cmp(a),
                        }
                    } else {
                        let a: f64 = a.parse().unwrap_or_default();
                        let b: f64 = b.parse().unwrap_or_default();
                        match self.sorting.order {
                            Order::Ascending => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
                            Order::Descending => b.partial_cmp(&a).unwrap_or(Ordering::Equal),
                        }
                    }
                } else {
                    Ordering::Equal
                }
            });
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
