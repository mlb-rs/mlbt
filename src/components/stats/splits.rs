use mlbt_api::stats::{Split, Stat};
use std::collections::HashSet;

pub struct StatSplits {
    pub season: Vec<Split>,
    pub year_by_year: Vec<Split>,
    pub career: Vec<Split>,
    pub game_log: Vec<Split>,
}

impl StatSplits {
    const RECENT_GAME_LOG_COUNT: usize = 15;

    pub(crate) fn from_stats(stats: Vec<Stat>) -> Self {
        let mut season = Vec::new();
        let mut year_by_year = Vec::new();
        let mut career = Vec::new();
        let mut game_log = Vec::new();

        for stat in stats {
            match stat.stat_type.display_name.as_str() {
                "season" => season = stat.splits,
                "yearByYear" => year_by_year = stat.splits,
                "career" => career = stat.splits,
                "gameLog" => game_log = stat.splits,
                _ => {}
            }
        }

        filter_combined_splits(&mut year_by_year);

        // only keep the most recent games
        let total = game_log.len();
        if total > Self::RECENT_GAME_LOG_COUNT {
            game_log.drain(..total - Self::RECENT_GAME_LOG_COUNT);
        }

        Self {
            season,
            year_by_year,
            career,
            game_log,
        }
    }
}

/// Remove combined season totals from yearByYear stats. When a player is traded mid-season, the API
/// includes per-team splits plus a combined split with no team for that season.
fn filter_combined_splits(splits: &mut Vec<Split>) {
    let mut seasons_with_team = HashSet::new();
    for split in splits.iter() {
        if split.team.is_some()
            && let Some(season) = &split.season
        {
            seasons_with_team.insert(season.clone());
        }
    }

    splits.retain(|s| {
        s.team.is_some() || !seasons_with_team.contains(s.season.as_deref().unwrap_or_default())
    });
}
