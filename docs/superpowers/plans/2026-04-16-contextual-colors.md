# Contextual Stat Colors Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the existing green→amber→red stat coloring system to OBP, SLG, OPS, and WHIP across all views, add multi-hit/HR highlights to the player game log, add W/L coloring to the team page schedule, and apply ERA coloring in the probable pitchers panel.

**Architecture:** All new color logic lives as `Option<Color>` functions in `src/components/util.rs` alongside `avg_color` and `era_color`. Each call site falls back to `Color::White` on `None` (average range). The team page schedule requires adding an `is_win: Option<bool>` field to `TeamGame` so the UI can color the score cell without re-parsing the string. The probable pitchers panel requires changing `to_row_cells` from `Vec<String>` to `Vec<Cell>` to support styling.

**Tech Stack:** Rust, ratatui/tui, existing `Theme` constants (`EXCELLENT`, `GOOD`, `BELOW_AVG`, `POOR`, `DIMMED`)

---

## Files Modified

- `src/components/util.rs` — add `obp_color`, `slg_color`, `ops_color`, `whip_color`
- `src/ui/stats.rs` — extend column-index coloring to OBP, SLG, OPS, WHIP
- `src/components/stats/player_profile.rs` — apply new color fns to season stats, splits, game log
- `src/components/team_page.rs` — add `is_win: Option<bool>` to `TeamGame`
- `src/ui/team_page.rs` — color score cell green/red based on `is_win`
- `src/components/probable_pitchers.rs` — change `to_row_cells` to return `Vec<Cell>`, apply `era_color`
- `src/ui/probable_pitchers.rs` — update call site for `Vec<Cell>`

---

### Task 1: Add color utility functions

**Files:**
- Modify: `src/components/util.rs`

- [ ] **Step 1: Add the four new color functions after `win_pct_color`**

```rust
/// Color for an OBP stat string. Returns `None` for average range (.290–.349).
pub(crate) fn obp_color(obp: &str) -> Option<Color> {
    obp.parse::<f64>().ok().and_then(|v| {
        if v == 0.0 {
            Some(Theme::DIMMED)
        } else if v >= 0.380 {
            Some(Theme::EXCELLENT)
        } else if v >= 0.350 {
            Some(Theme::GOOD)
        } else if v < 0.250 {
            Some(Theme::POOR)
        } else if v < 0.290 {
            Some(Theme::BELOW_AVG)
        } else {
            None
        }
    })
}

/// Color for a SLG stat string. Returns `None` for average range (.350–.449).
pub(crate) fn slg_color(slg: &str) -> Option<Color> {
    slg.parse::<f64>().ok().and_then(|v| {
        if v == 0.0 {
            Some(Theme::DIMMED)
        } else if v >= 0.500 {
            Some(Theme::EXCELLENT)
        } else if v >= 0.450 {
            Some(Theme::GOOD)
        } else if v < 0.300 {
            Some(Theme::POOR)
        } else if v < 0.350 {
            Some(Theme::BELOW_AVG)
        } else {
            None
        }
    })
}

/// Color for an OPS stat string. Returns `None` for average range (.680–.799).
pub(crate) fn ops_color(ops: &str) -> Option<Color> {
    ops.parse::<f64>().ok().and_then(|v| {
        if v == 0.0 {
            Some(Theme::DIMMED)
        } else if v >= 0.900 {
            Some(Theme::EXCELLENT)
        } else if v >= 0.800 {
            Some(Theme::GOOD)
        } else if v < 0.600 {
            Some(Theme::POOR)
        } else if v < 0.680 {
            Some(Theme::BELOW_AVG)
        } else {
            None
        }
    })
}

/// Color for a WHIP stat string. Returns `None` for average range (1.11–1.39).
/// Lower WHIP is better — color scale is inverted relative to batting stats.
pub(crate) fn whip_color(whip: &str) -> Option<Color> {
    whip.parse::<f64>().ok().and_then(|v| {
        if v == 0.0 {
            Some(Theme::DIMMED)
        } else if v <= 0.90 {
            Some(Theme::EXCELLENT)
        } else if v <= 1.10 {
            Some(Theme::GOOD)
        } else if v >= 1.60 {
            Some(Theme::POOR)
        } else if v >= 1.40 {
            Some(Theme::BELOW_AVG)
        } else {
            None
        }
    })
}
```

- [ ] **Step 2: Add unit tests at the bottom of `src/components/util.rs`**

```rust
#[cfg(test)]
mod stat_color_tests {
    use super::*;

    #[test]
    fn obp_excellent() { assert_eq!(obp_color(".400"), Some(Theme::EXCELLENT)); }
    #[test]
    fn obp_good() { assert_eq!(obp_color(".355"), Some(Theme::GOOD)); }
    #[test]
    fn obp_average() { assert_eq!(obp_color(".310"), None); }
    #[test]
    fn obp_below() { assert_eq!(obp_color(".270"), Some(Theme::BELOW_AVG)); }
    #[test]
    fn obp_poor() { assert_eq!(obp_color(".240"), Some(Theme::POOR)); }
    #[test]
    fn obp_zero() { assert_eq!(obp_color(".000"), Some(Theme::DIMMED)); }

    #[test]
    fn slg_excellent() { assert_eq!(slg_color(".520"), Some(Theme::EXCELLENT)); }
    #[test]
    fn slg_good() { assert_eq!(slg_color(".460"), Some(Theme::GOOD)); }
    #[test]
    fn slg_average() { assert_eq!(slg_color(".400"), None); }
    #[test]
    fn slg_below() { assert_eq!(slg_color(".320"), Some(Theme::BELOW_AVG)); }
    #[test]
    fn slg_poor() { assert_eq!(slg_color(".280"), Some(Theme::POOR)); }

    #[test]
    fn ops_excellent() { assert_eq!(ops_color(".950"), Some(Theme::EXCELLENT)); }
    #[test]
    fn ops_good() { assert_eq!(ops_color(".820"), Some(Theme::GOOD)); }
    #[test]
    fn ops_average() { assert_eq!(ops_color(".730"), None); }
    #[test]
    fn ops_below() { assert_eq!(ops_color(".640"), Some(Theme::BELOW_AVG)); }
    #[test]
    fn ops_poor() { assert_eq!(ops_color(".550"), Some(Theme::POOR)); }

    #[test]
    fn whip_excellent() { assert_eq!(whip_color("0.85"), Some(Theme::EXCELLENT)); }
    #[test]
    fn whip_good() { assert_eq!(whip_color("1.05"), Some(Theme::GOOD)); }
    #[test]
    fn whip_average() { assert_eq!(whip_color("1.25"), None); }
    #[test]
    fn whip_below() { assert_eq!(whip_color("1.45"), Some(Theme::BELOW_AVG)); }
    #[test]
    fn whip_poor() { assert_eq!(whip_color("1.70"), Some(Theme::POOR)); }
    #[test]
    fn whip_zero() { assert_eq!(whip_color("0.00"), Some(Theme::DIMMED)); }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test -p mlbtg stat_color_tests 2>&1 | grep -E "test .* (ok|FAILED|error)"
```

Expected: all 22 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/components/util.rs
git commit -m "feat: add obp_color, slg_color, ops_color, whip_color utility functions"
```

---

### Task 2: Stats table — color OBP, SLG, OPS, WHIP columns

**Files:**
- Modify: `src/ui/stats.rs`

- [ ] **Step 1: Import new color functions**

In `src/ui/stats.rs`, find the import line:
```rust
use crate::components::util::{DimColor, avg_color, era_color};
```
Replace with:
```rust
use crate::components::util::{DimColor, avg_color, era_color, obp_color, ops_color, slg_color, whip_color};
```

- [ ] **Step 2: Add index variables and column matching**

Find the loop that scans headers (around line 24):
```rust
let mut avg_idx = None;
let mut era_idx = None;
for (i, name) in header.iter().enumerate() {
    match name.as_str() {
        "AVG" => avg_idx = Some(i),
        "ERA" => era_idx = Some(i),
        _ => {}
    }
}
```
Replace with:
```rust
let mut avg_idx = None;
let mut era_idx = None;
let mut obp_idx = None;
let mut slg_idx = None;
let mut ops_idx = None;
let mut whip_idx = None;
for (i, name) in header.iter().enumerate() {
    match name.as_str() {
        "AVG" => avg_idx = Some(i),
        "ERA" => era_idx = Some(i),
        "OBP" => obp_idx = Some(i),
        "SLG" => slg_idx = Some(i),
        "OPS" => ops_idx = Some(i),
        "WHIP" => whip_idx = Some(i),
        _ => {}
    }
}
```

- [ ] **Step 3: Apply colors in the cell-building loop**

Find the per-cell color logic (around line 65):
```rust
let color = if Some(i) == avg_idx {
    avg_color(cell).unwrap_or(Color::White)
} else if Some(i) == era_idx {
    era_color(cell).unwrap_or(Color::White)
} else {
    cell.as_str().dim_or(Color::White)
};
```
Replace with:
```rust
let color = if Some(i) == avg_idx {
    avg_color(cell).unwrap_or(Color::White)
} else if Some(i) == era_idx {
    era_color(cell).unwrap_or(Color::White)
} else if Some(i) == obp_idx {
    obp_color(cell).unwrap_or(Color::White)
} else if Some(i) == slg_idx {
    slg_color(cell).unwrap_or(Color::White)
} else if Some(i) == ops_idx {
    ops_color(cell).unwrap_or(Color::White)
} else if Some(i) == whip_idx {
    whip_color(cell).unwrap_or(Color::White)
} else {
    cell.as_str().dim_or(Color::White)
};
```

- [ ] **Step 4: Verify compile**

```bash
cargo check 2>&1 | grep "^error"
```
Expected: no output.

- [ ] **Step 5: Commit**

```bash
git add src/ui/stats.rs
git commit -m "feat: color OBP, SLG, OPS, WHIP columns in stats table"
```

---

### Task 3: Player profile — season stats OBP, SLG, OPS, WHIP

**Files:**
- Modify: `src/components/stats/player_profile.rs`

- [ ] **Step 1: Import new color functions**

Find the import line in `src/components/stats/player_profile.rs`:
```rust
use crate::components::util::{
    DimColor, OptionDisplayExt, OptionMapDisplayExt, avg_color, era_color, format_date,
};
```
Replace with:
```rust
use crate::components::util::{
    DimColor, OptionDisplayExt, OptionMapDisplayExt, avg_color, era_color, format_date,
    obp_color, ops_color, slg_color, whip_color,
};
```

- [ ] **Step 2: Color OBP, SLG, OPS in season hitting stats**

In `season_stat_cells` (the hitting branch, around lines 175–177), find:
```rust
s.obp.as_str().into(),
s.slg.as_str().into(),
s.ops.as_str().into(),
```
Replace with:
```rust
Cell::from(s.obp.as_str()).fg(obp_color(s.obp.as_str()).unwrap_or(Color::White)),
Cell::from(s.slg.as_str()).fg(slg_color(s.slg.as_str()).unwrap_or(Color::White)),
Cell::from(s.ops.as_str()).fg(ops_color(s.ops.as_str()).unwrap_or(Color::White)),
```

- [ ] **Step 3: Color WHIP in season pitching stats**

In the same function, pitching branch (around line 210), find:
```rust
s.whip.as_str().into(),
```
Replace with:
```rust
Cell::from(s.whip.as_str()).fg(whip_color(s.whip.as_str()).unwrap_or(Color::White)),
```

- [ ] **Step 4: Verify compile**

```bash
cargo check 2>&1 | grep "^error"
```
Expected: no output.

- [ ] **Step 5: Commit**

```bash
git add src/components/stats/player_profile.rs
git commit -m "feat: color OBP, SLG, OPS, WHIP in player profile season stats"
```

---

### Task 4: Player profile — splits OBP, SLG, WHIP

**Files:**
- Modify: `src/components/stats/player_profile.rs`

- [ ] **Step 1: Color OBP and SLG in recent splits hitting rows**

In `build_splits_rows`, inside the `Some(RecentStats::Hitting(s))` branch (around lines 386–387), find:
```rust
s.obp.as_str().into(),
s.slg.as_str().into(),
```
Replace with:
```rust
Cell::from(s.obp.as_str()).fg(obp_color(s.obp.as_str()).unwrap_or(Color::White)),
Cell::from(s.slg.as_str()).fg(slg_color(s.slg.as_str()).unwrap_or(Color::White)),
```

- [ ] **Step 2: Color WHIP in recent splits pitching rows**

In the `Some(RecentStats::Pitching(s))` branch (around line 404), find:
```rust
s.whip.as_str().into(),
```
Replace with:
```rust
Cell::from(s.whip.as_str()).fg(whip_color(s.whip.as_str()).unwrap_or(Color::White)),
```

- [ ] **Step 3: Verify compile**

```bash
cargo check 2>&1 | grep "^error"
```
Expected: no output.

- [ ] **Step 4: Commit**

```bash
git add src/components/stats/player_profile.rs
git commit -m "feat: color OBP, SLG, WHIP in player profile splits"
```

---

### Task 5: Player profile — game log multi-hit and HR highlights

**Files:**
- Modify: `src/components/stats/player_profile.rs`

The game log hitting row already has `s.hits` and `s.home_runs` as integers in `game_log_cells`. Color `hits` amber (GOOD) when ≥ 3, and `home_runs` amber when > 0. These are contextual highlights — a multi-hit game or a HR is notable regardless of season totals.

- [ ] **Step 1: Highlight hits and home runs in game log hitting branch**

In `game_log_cells`, inside the `StatSplit::Hitting(s)` branch (around lines 246–252), find:
```rust
Cell::from(s.hits.to_string()).fg(s.hits.dim_or(Color::White)),
Cell::from(s.doubles.to_string()).fg(s.doubles.dim_or(Color::White)),
Cell::from(s.triples.to_string()).fg(s.triples.dim_or(Color::White)),
Cell::from(s.home_runs.to_string()).fg(s.home_runs.dim_or(Color::White)),
```
Replace with:
```rust
Cell::from(s.hits.to_string()).fg(if s.hits >= 3 {
    Theme::GOOD
} else {
    s.hits.dim_or(Color::White)
}),
Cell::from(s.doubles.to_string()).fg(s.doubles.dim_or(Color::White)),
Cell::from(s.triples.to_string()).fg(s.triples.dim_or(Color::White)),
Cell::from(s.home_runs.to_string()).fg(if s.home_runs > 0 {
    Theme::GOOD
} else {
    Theme::DIMMED
}),
```

Also add the `Theme` import at the top of the file if not already present:
```rust
use crate::theme::Theme;
```

- [ ] **Step 2: Verify compile**

```bash
cargo check 2>&1 | grep "^error"
```
Expected: no output.

- [ ] **Step 3: Commit**

```bash
git add src/components/stats/player_profile.rs
git commit -m "feat: highlight multi-hit games and home runs in player game log"
```

---

### Task 6: Team page schedule — W/L coloring

**Files:**
- Modify: `src/components/team_page.rs`
- Modify: `src/ui/team_page.rs`

The W/L result is currently baked into `time_or_score` as a string like `"5-2 W"`. Rather than parsing that string, add an `is_win: Option<bool>` field to `TeamGame` — `None` for future/in-progress games, `Some(true)` for wins, `Some(false)` for losses.

- [ ] **Step 1: Add `is_win` field to `TeamGame`**

In `src/components/team_page.rs`, find the struct:
```rust
pub struct TeamGame {
    pub date: NaiveDate,
    pub date_display: String,
    pub opponent: String,
    pub time_or_score: String,
    pub start_time_utc: Option<DateTime<Utc>>,
    pub is_home: bool,
    pub is_past: bool,
```
Add `is_win` after `is_past`:
```rust
pub struct TeamGame {
    pub date: NaiveDate,
    pub date_display: String,
    pub opponent: String,
    pub time_or_score: String,
    pub start_time_utc: Option<DateTime<Utc>>,
    pub is_home: bool,
    pub is_past: bool,
    pub is_win: Option<bool>,
```

- [ ] **Step 2: Populate `is_win` when building `TeamGame` instances**

In the `TeamGame::from_schedule` (or equivalent builder) function, find where `time_or_score` is set. The existing code already computes `result` as `"W"`, `"L"`, or `"T"`. Find the `games.push(TeamGame { ... })` call (around line 109) and add `is_win`:

```rust
let is_win = if is_past {
    Some(team_score > opp_score)
} else {
    None
};
```

Then in `games.push(TeamGame { ... })`, add:
```rust
is_win,
```

- [ ] **Step 3: Color the score cell in `src/ui/team_page.rs`**

Find `style_schedule_game` (around line 333):
```rust
fn style_schedule_game(today: NaiveDate, g: &TeamGame) -> (Style, Style) {
    let date_style = if g.date == today {
        TODAY_STYLE
    } else if g.is_past {
        PAST_STYLE
    } else if g.is_home {
        HOME_STYLE
    } else {
        AWAY_STYLE
    };
    let text_style = if g.is_past {
        PAST_STYLE
    } else {
        Style::default()
    };
    (date_style, text_style)
}
```

This function returns a single `text_style` used for both opponent and score. Split them: return a third `score_style` for the score cell.

Replace the function and its call site:

```rust
fn style_schedule_game(today: NaiveDate, g: &TeamGame) -> (Style, Style, Style) {
    let date_style = if g.date == today {
        TODAY_STYLE
    } else if g.is_past {
        PAST_STYLE
    } else if g.is_home {
        HOME_STYLE
    } else {
        AWAY_STYLE
    };
    let text_style = if g.is_past {
        PAST_STYLE
    } else {
        Style::default()
    };
    let score_style = match g.is_win {
        Some(true) => Style::new().fg(Color::Green),
        Some(false) => Style::new().fg(Color::Red),
        None => text_style,
    };
    (date_style, text_style, score_style)
}
```

In `render_schedule`, update the call site (around line 238):
```rust
let (date_style, text_style) = style_schedule_game(self.state.date, g);
Row::new(vec![
    Cell::from(Span::styled(g.date_display.as_str(), date_style)),
    Cell::from(Span::styled(g.opponent.as_str(), text_style)),
    Cell::from(Span::styled(g.time_or_score.as_str(), text_style)),
])
```
Replace with:
```rust
let (date_style, text_style, score_style) = style_schedule_game(self.state.date, g);
Row::new(vec![
    Cell::from(Span::styled(g.date_display.as_str(), date_style)),
    Cell::from(Span::styled(g.opponent.as_str(), text_style)),
    Cell::from(Span::styled(g.time_or_score.as_str(), score_style)),
])
```

- [ ] **Step 4: Verify compile**

```bash
cargo check 2>&1 | grep "^error"
```
Expected: no output.

- [ ] **Step 5: Commit**

```bash
git add src/components/team_page.rs src/ui/team_page.rs
git commit -m "feat: color W/L green/red in team page schedule"
```

---

### Task 7: Probable pitchers — ERA coloring

**Files:**
- Modify: `src/components/probable_pitchers.rs`
- Modify: `src/ui/probable_pitchers.rs`

`to_row_cells` currently returns `Vec<String>`, which can't carry color. Change it to `Vec<Cell<'static>>` and apply `era_color` to the ERA cell.

- [ ] **Step 1: Update `to_row_cells` in `src/components/probable_pitchers.rs`**

Add imports at the top of the file:
```rust
use crate::components::util::{era_color, OptionDisplayExt};
use tui::style::Color;
use tui::widgets::Cell;
```

Replace the `to_row_cells` method:
```rust
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
```

- [ ] **Step 2: Update call sites in `src/ui/probable_pitchers.rs`**

Find (around line 35):
```rust
let away_row = Row::new(self.matchup.away_pitcher.to_row_cells(away_team));
let home_row = Row::new(self.matchup.home_pitcher.to_row_cells(home_team));
```

These lines don't need changes — `Row::new` accepts `Vec<Cell>` the same as `Vec<String>`. Just verify it compiles.

- [ ] **Step 3: Verify compile**

```bash
cargo check 2>&1 | grep "^error"
```
Expected: no output.

- [ ] **Step 4: Commit**

```bash
git add src/components/probable_pitchers.rs src/ui/probable_pitchers.rs
git commit -m "feat: color ERA in probable pitchers panel"
```

---

### Task 8: Push and verify CI

- [ ] **Step 1: Run all tests locally**

```bash
cargo test 2>&1 | tail -5
```
Expected: `test result: ok.`

- [ ] **Step 2: Push branch**

```bash
git push -u origin feat/colors
```

- [ ] **Step 3: Watch CI**

```bash
gh run list --repo agiacalone/mlbtg --limit 3 --workflow CI
```
Expected: latest run shows `success`.
