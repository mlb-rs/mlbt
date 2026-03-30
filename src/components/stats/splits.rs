use mlbt_api::stats::{Split, Stat, StatSplit};
use std::collections::HashSet;

pub struct StatSplits {
    pub season: Vec<Split>,
    pub year_by_year: Vec<Split>,
    pub career: Vec<Split>,
    pub game_log: Vec<Split>,
    pub recent_splits: Vec<RecentSplit>,
}

pub struct RecentSplit {
    pub label: &'static str,
    pub stat: Option<RecentStats>,
}

pub enum RecentStats {
    Hitting(HittingTotals),
    Pitching(PitchingTotals),
}

pub struct HittingTotals {
    pub ab: u16,
    pub r: u16,
    pub h: u16,
    pub hr: u16,
    pub rbi: u16,
    pub bb: u16,
    pub so: u16,
    pub sb: u16,
    pub avg: String,
    pub obp: String,
    pub slg: String,
}

pub struct PitchingTotals {
    pub w: u16,
    pub l: u16,
    pub era: String,
    pub g: u16,
    pub gs: u16,
    pub sv: u16,
    pub ip: String,
    pub h: u16,
    pub er: u16,
    pub bb: u16,
    pub so: u16,
    pub whip: String,
}

const SPLIT_WINDOWS: &[(usize, &str)] = &[
    (7, "Last 7 Games"),
    (15, "Last 15 Games"),
    (30, "Last 30 Games"),
];

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

        let recent_splits = compute_recent_splits(&game_log);

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
            recent_splits,
        }
    }
}

/// Compute the stats for the most recent games in the game log. For the last 7 games if there are
/// less than 7 games, compute the stats for the available games. If less than 15 or 30, don't show
/// data.
fn compute_recent_splits(game_log: &[Split]) -> Vec<RecentSplit> {
    let total = game_log.len();
    SPLIT_WINDOWS
        .iter()
        .map(|&(n, label)| {
            let stat = if total > 0 && (n == 7 || total >= n) {
                let start = total.saturating_sub(n);
                aggregate_stats(&game_log[start..])
            } else {
                None
            };
            RecentSplit { label, stat }
        })
        .collect()
}

fn aggregate_stats(splits: &[Split]) -> Option<RecentStats> {
    let first = splits.first()?;
    match &first.stat {
        StatSplit::Hitting(_) => Some(RecentStats::Hitting(aggregate_hitting(splits))),
        StatSplit::Pitching(_) => Some(RecentStats::Pitching(aggregate_pitching(splits))),
    }
}

fn aggregate_hitting(splits: &[Split]) -> HittingTotals {
    let (mut ab, mut r, mut h, mut hr, mut rbi) = (0u16, 0u16, 0u16, 0u16, 0u16);
    let (mut bb, mut so, mut sb) = (0u16, 0u16, 0u16);
    let (mut hbp, mut sf, mut tb) = (0u16, 0u16, 0u16);

    for split in splits {
        if let StatSplit::Hitting(s) = &split.stat {
            ab += s.at_bats;
            r += s.runs;
            h += s.hits;
            hr += s.home_runs;
            rbi += s.rbi;
            bb += s.base_on_balls;
            so += s.strike_outs;
            sb += s.stolen_bases;
            hbp += s.hit_by_pitch;
            sf += s.sac_flies;
            tb += s.total_bases;
        }
    }

    HittingTotals {
        ab,
        r,
        h,
        hr,
        rbi,
        bb,
        so,
        sb,
        avg: format_rate(h, ab),
        obp: format_rate(h + bb + hbp, ab + bb + hbp + sf),
        slg: format_rate(tb, ab),
    }
}

fn aggregate_pitching(splits: &[Split]) -> PitchingTotals {
    let (mut w, mut l, mut g, mut gs, mut sv) = (0u16, 0u16, 0u16, 0u16, 0u16);
    let (mut h, mut er, mut bb, mut so) = (0u16, 0u16, 0u16, 0u16);
    let mut total_outs: u32 = 0;

    for split in splits {
        if let StatSplit::Pitching(s) = &split.stat {
            w += s.wins;
            l += s.losses;
            g += s.games_played;
            gs += s.games_started;
            sv += s.saves;
            h += s.hits;
            er += s.earned_runs;
            bb += s.base_on_balls;
            so += s.strike_outs;
            total_outs += parse_ip_to_outs(&s.innings_pitched);
        }
    }

    let era = if total_outs > 0 {
        format!("{:.2}", er as f64 * 27.0 / total_outs as f64)
    } else {
        "---".to_string()
    };
    let whip = if total_outs > 0 {
        format!("{:.2}", (bb + h) as f64 * 3.0 / total_outs as f64)
    } else {
        "---".to_string()
    };

    PitchingTotals {
        w,
        l,
        era,
        g,
        gs,
        sv,
        ip: format_outs_to_ip(total_outs),
        h,
        er,
        bb,
        so,
        whip,
    }
}

/// Format a rate stat like AVG/OBP/SLG (e.g., ".345", "1.000").
fn format_rate(numerator: u16, denominator: u16) -> String {
    if denominator == 0 {
        return "---".to_string();
    }
    let value = numerator as f64 / denominator as f64;
    if value >= 1.0 {
        format!("{value:.3}")
    } else {
        let s = format!("{value:.3}");
        s.strip_prefix('0').unwrap_or(&s).to_string()
    }
}

fn parse_ip_to_outs(ip: &str) -> u32 {
    let (innings, partial) = ip.split_once('.').unwrap_or((ip, "0"));
    innings.parse::<u32>().unwrap_or(0) * 3 + partial.parse::<u32>().unwrap_or(0)
}

fn format_outs_to_ip(total_outs: u32) -> String {
    format!("{}.{}", total_outs / 3, total_outs % 3)
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
