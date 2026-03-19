use crate::components::constants::lookup_team;
use indexmap::IndexMap;
use mlbt_api::client::StatGroup;
use mlbt_api::stats::{HittingStat, PitchingStat, StatSplit, StatsResponse};
use std::cmp::Ordering;
use std::string::ToString;
use std::sync::Arc;

/// The width of the first column, which is a longer item like team name.
pub const STATS_FIRST_COL_WIDTH: u16 = 28;
/// The width of normal columns.
pub const STATS_DEFAULT_COL_WIDTH: u16 = 6;
pub const PLAYER_COLUMN_NAME: &str = "Player";
pub const TEAM_COLUMN_NAME: &str = "Team";

/// Table data in row oriented form: (header, ids, rows).
/// `ids` are the player/team id for each row.
pub type TableData = (Vec<String>, Vec<u64>, Vec<Vec<String>>);

/// Stores whether a team/player and pitching/hitting stat should be viewed.
#[derive(Clone, Copy, Debug)]
pub struct StatType {
    pub group: StatGroup,
    pub team_player: TeamOrPlayer,
}

impl StatType {
    pub fn search_label(&self) -> &'static str {
        match self.team_player {
            TeamOrPlayer::Team => "teams",
            TeamOrPlayer::Player => match self.group {
                StatGroup::Hitting => "hitters",
                StatGroup::Pitching => "pitchers",
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

/// The information for a stat, including all the data values.
pub struct TableEntry {
    /// Longer description of the stat to be displayed in the options toggle pane.
    pub description: String,
    /// Whether to display the column or not.
    pub active: bool,
    /// The data values. Note they are stored as strings to allow for creation of a tui-rs Cell.
    pub rows: Vec<String>,
}

/// Table data and cached generation results. Owned by `StatsState`.
pub struct StatsTable {
    /// Stores the data in columnar form. The key is the column name and the value contains the
    /// data stored as a vector. `IndexMap` is used to store the data in inserted order, which
    /// enables deterministic access of the data (for transforming to row oriented).
    pub columns: IndexMap<String, TableEntry>,
    /// The column and direction used for sorting the stats.
    pub sorting: Sort,
    /// Cached table data.
    cache: Option<Arc<TableData>>,
    /// Row ids (player or team) parallel to the columnar data.
    row_ids: Vec<u64>,
}

impl Default for StatsTable {
    fn default() -> Self {
        Self {
            columns: IndexMap::new(),
            sorting: Sort::default(),
            cache: None,
            row_ids: Vec::new(),
        }
    }
}

impl StatsTable {
    pub fn invalidate_cache(&mut self) {
        self.cache = None;
    }

    pub fn cached(&self) -> Option<&Arc<TableData>> {
        self.cache.as_ref()
    }

    pub fn load(&mut self, stats: &StatsResponse, team_player: TeamOrPlayer) {
        self.columns.clear();
        self.row_ids.clear();
        self.invalidate_cache();
        for stat in &stats.stats {
            for split in &stat.splits {
                let team_name = split
                    .team
                    .as_ref()
                    .map(|t| t.name.clone())
                    .unwrap_or_default();
                let team = lookup_team(&team_name);
                let team_abbreviation = Some(team.abbreviation.to_string());
                let (name, id) = match &split.player {
                    Some(p) => (p.full_name.clone(), p.id),
                    None => (team_name, team.id as u64),
                };
                self.row_ids.push(id);
                match &split.stat {
                    StatSplit::Pitching(s) => {
                        self.load_pitching_stats(name, team_abbreviation, s, team_player)
                    }
                    StatSplit::Hitting(s) => {
                        self.load_hitting_stats(name, team_abbreviation, s, team_player)
                    }
                };
            }
        }
    }

    /// Create the header and the table rows from the table map. Basically transforms from columnar
    /// to row based, filtering on whether the data is active.
    ///
    /// NOTE: The cache does not key on `filter`. Callers must call `invalidate_cache()` whenever
    /// the filter changes (e.g. when search matches are updated or cleared).
    pub fn generate(&mut self, filter: Option<&[usize]>) -> Arc<TableData> {
        if let Some(cached) = &self.cache {
            return cached.clone();
        }
        let data = Arc::new(self.rebuild_table(filter));
        self.cache = Some(data.clone());
        data
    }

    fn rebuild_table(&self, filter: Option<&[usize]>) -> TableData {
        if self.columns.is_empty() {
            return (vec![], vec![], vec![vec![]]);
        }

        // get the number of rows, which might be zero while data is loading
        let len = match self.columns.first() {
            Some((_, v)) => v.rows.len(),
            None => 0,
        };
        if len == 0 {
            return (vec![], vec![], vec![vec![]]);
        }

        // determine how many rows we need to render
        let row_count = match filter {
            Some(indices) => indices.len(),
            None => len,
        };

        let mut rows = vec![Vec::with_capacity(self.columns.len()); row_count];
        let mut header = Vec::with_capacity(self.columns.len());

        // build ids parallel to rows, applying the same filter
        let mut ids: Vec<u64> = match filter {
            Some(indices) => indices
                .iter()
                .filter_map(|&i| self.row_ids.get(i).copied())
                .collect(),
            None => self.row_ids.clone(),
        };

        // access the data in stored order because of `IndexMap` and only clone rows that will be
        // actually displayed
        for (key, col) in &self.columns {
            if col.active {
                header.push(key.clone());

                if let Some(indices) = filter {
                    for (out_idx, &src_idx) in indices.iter().enumerate() {
                        if let Some(val) = col.rows.get(src_idx) {
                            rows[out_idx].push(val.clone());
                        }
                    }
                } else {
                    for (idx, val) in col.rows.iter().enumerate() {
                        rows[idx].push(val.clone());
                    }
                }
            }
        }

        self.sort_rows(&mut rows, &mut ids);
        (header, ids, rows)
    }

    /// Insert stats into the table map. If the key isn't present a new column is created, otherwise
    /// the data is simply added.
    fn table_helper<T>(&mut self, key: &str, description: &str, active: bool, value: T)
    where
        T: ToString,
    {
        self.columns
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
    fn load_pitching_stats(
        &mut self,
        name: String,
        team_abbreviation: Option<String>,
        stat: &PitchingStat,
        team_player: TeamOrPlayer,
    ) {
        self.format_name_columns(name, team_abbreviation, team_player);
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
    fn load_hitting_stats(
        &mut self,
        name: String,
        team_abbreviation: Option<String>,
        stat: &HittingStat,
        team_player: TeamOrPlayer,
    ) {
        self.format_name_columns(name, team_abbreviation, team_player);
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

    fn format_name_columns(
        &mut self,
        name: String,
        team_abbreviation: Option<String>,
        team_player: TeamOrPlayer,
    ) {
        match team_player {
            TeamOrPlayer::Team => {
                self.table_helper(TEAM_COLUMN_NAME, "", true, name);
            }
            TeamOrPlayer::Player => {
                // show the team abbreviation if it exists next to the player name
                self.table_helper(PLAYER_COLUMN_NAME, "", true, name);
                if let Some(abb) = team_abbreviation {
                    self.table_helper(TEAM_COLUMN_NAME, "", true, abb);
                }
            }
        };
    }

    /// Deactivate columns that would overflow the available width.
    pub fn trim_columns(&mut self, available_width: u16) {
        // Get the indices of active columns
        let mut active: Vec<usize> = self
            .columns
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
        let mut changed = false;
        while column_width >= available_width && !active.is_empty() {
            let key = active.pop().unwrap();
            if let Some((_, v)) = self.columns.get_index_mut(key) {
                v.active = false;
                column_width -= STATS_DEFAULT_COL_WIDTH;
                changed = true;
            }
        }
        if changed {
            self.invalidate_cache();
        }
    }

    /// Toggle the visibility of the stat column that is selected.
    pub fn toggle_stat(&mut self, selected_index: usize) {
        self.invalidate_cache();
        let sort_column_index = self.get_sort_column_index();

        if let Some((_, v)) = self.columns.get_index_mut(selected_index) {
            v.active = !v.active;
            // if the column is the sort column and it's toggled off, reset the sorting
            if sort_column_index.is_some_and(|idx| idx == selected_index) && !v.active {
                self.sorting.column_name = None;
            }
        }
    }

    /// Get the index of the sort column while taking into account if columns to the left of it are
    /// active.
    fn get_sort_column_index(&self) -> Option<usize> {
        let sort_column = self.sorting.column_name.as_ref()?;

        let mut active_idx = 0;
        for (column_name, entry) in self.columns.iter() {
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
    pub fn store_sort_column(&mut self, selected_index: usize) {
        self.invalidate_cache();

        // if the stat isn't already active don't sort
        if let Some((column_name, entry)) = self.columns.get_index(selected_index) {
            if !entry.active {
                return;
            }
            self.sorting = Sort {
                column_name: Some(column_name.clone()),
                order: !self.sorting.order,
            };
        }
    }

    /// Sort rows and ids together by the selected stat.
    fn sort_rows(&self, rows: &mut Vec<Vec<String>>, ids: &mut Vec<u64>) {
        let sort_column_index = self.get_sort_column_index();
        let sort_column_name = self.sorting.column_name.as_ref();

        if let (Some(sort_column_index), Some(sort_column)) = (sort_column_index, sort_column_name)
        {
            let mut ordering: Vec<usize> = (0..rows.len()).collect();
            ordering.sort_by(|&a, &b| {
                let a = rows[a].get(sort_column_index);
                let b = rows[b].get(sort_column_index);
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

            *rows = ordering.iter().map(|&i| rows[i].clone()).collect();
            *ids = ordering.iter().map(|&i| ids[i]).collect();
        }
    }

    /// Returns the total number of data rows.
    pub fn total_row_count(&self) -> usize {
        self.columns
            .values()
            .next()
            .map(|entry| entry.rows.len())
            .unwrap_or(0)
    }
}
