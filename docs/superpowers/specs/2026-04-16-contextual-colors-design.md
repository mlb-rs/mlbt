---
title: Contextual Stat Colors
date: 2026-04-16
status: approved
---

# Contextual Stat Colors

## Goal

Extend the existing green→amber→red stat coloring system to all meaningful baseball metrics across every view where they appear. The principle: glance at the screen and immediately know whether a number is good or bad — no reading required.

## Background

The app already has a working color vocabulary:
- `Theme::EXCELLENT` (green) — elite
- `Theme::GOOD` (amber) — above average
- `Theme::BELOW_AVG` (orange) — below average
- `Theme::POOR` (red) — poor
- `Theme::DIMMED` (gray) — zero / empty
- White — average (no color applied)

`avg_color()` and `era_color()` in `src/components/util.rs` implement this pattern and return `Option<Color>` (None = average range). All new color functions follow the same convention.

## New Color Functions (`src/components/util.rs`)

### `obp_color(obp: &str) -> Option<Color>`
| Range | Color |
|---|---|
| ≥ .380 | EXCELLENT |
| ≥ .350 | GOOD |
| .290–.349 | None (white) |
| .250–.289 | BELOW_AVG |
| < .250 | POOR |
| .000 | DIMMED |

### `slg_color(slg: &str) -> Option<Color>`
| Range | Color |
|---|---|
| ≥ .500 | EXCELLENT |
| ≥ .450 | GOOD |
| .350–.449 | None (white) |
| .300–.349 | BELOW_AVG |
| < .300 | POOR |
| .000 | DIMMED |

### `ops_color(ops: &str) -> Option<Color>`
| Range | Color |
|---|---|
| ≥ .900 | EXCELLENT |
| ≥ .800 | GOOD |
| .680–.799 | None (white) |
| .600–.679 | BELOW_AVG |
| < .600 | POOR |
| .000 | DIMMED |

### `whip_color(whip: &str) -> Option<Color>`
Lower is better — inverse of batting stats.
| Range | Color |
|---|---|
| ≤ 0.90 | EXCELLENT |
| ≤ 1.10 | GOOD |
| 1.11–1.39 | None (white) |
| 1.40–1.59 | BELOW_AVG |
| ≥ 1.60 | POOR |
| 0.00 / empty | DIMMED |

## Affected Views

### Stats Table (`src/ui/stats.rs`)
The column-index approach already handles AVG and ERA by position. Add OBP, SLG, OPS, WHIP to the same lookup — find their column indices by header name and apply the corresponding color function. The column names in the API data are `"obp"`, `"slg"`, `"ops"`, `"whip"`.

### Player Profile — Season Stats (`src/components/stats/player_profile.rs`)
`season_stat_cells()` builds hitting and pitching stat rows. Add `obp_color`, `slg_color`, `ops_color` to the hitting branch; `whip_color` to the pitching branch. These stats are already present in the `HittingStats` and `PitchingStats` structs.

### Player Profile — Splits (`src/components/stats/player_profile.rs`)
`build_splits_rows()` / `build_recent_splits_rows()` — same treatment as season stats. OBP, SLG, OPS for hitting splits; WHIP for pitching splits.

### Player Profile — Game Log (`src/components/stats/player_profile.rs`)
`game_log_cells()` — individual game performance. Apply:
- `avg_color` to AVG (already done)
- `era_color` to ERA (already done)
- For hitting: bold or amber highlight when HR > 0 or hits ≥ 3 in a single game (multi-hit)
- For pitching: `whip_color` to game WHIP if available, otherwise skip

### Boxscore Pitching (`src/components/boxscore.rs`)
ERA is already colored. WHIP is not stored in `PitcherRow` — skip for now.

### Team Page Schedule (`src/ui/team_page.rs`)
Past games already show date and opponent. Add W/L result coloring: green for W, red for L. Same pattern as player game log.

### Probable Pitchers (`src/components/probable_pitchers.rs` + `src/ui/probable_pitchers.rs`)
Apply `era_color` and `whip_color` to ERA and WHIP columns if they exist in the displayed data. Verify field availability before implementing.

## Implementation Notes

- All new functions live in `src/components/util.rs` alongside `avg_color` and `era_color`
- All return `Option<Color>` — None means average range, caller falls back to white
- No `Symbols` threading required — these are unconditional data-quality colors, not team-color-gated
- Game log multi-hit highlight (hits ≥ 3) and HR highlight are simple integer comparisons, no new utility function needed

## Out of Scope

- K%, BB%, FIP, wRC+ — not consistently available in current API responses
- Any color changes to the gameday at-bat pitch-by-pitch view
- Changes to the win probability chart colors
