# Nerd Fonts Support Design

**Date:** 2026-04-15  
**Status:** Approved  
**Context:** PR contribution to mlb-rs/mlbt — must follow existing project conventions exactly, no scope creep, cleanly revertable commits.

---

## Overview

Add opt-in Nerd Fonts support via a single `nerd_fonts = true` config toggle. When disabled (the default), the app is byte-for-byte identical to current behavior. When enabled, visual glyphs and team color theming are applied across the UI.

Delivered as four separate commits so the maintainer can cherry-pick or revert individual pieces.

---

## Config & Settings

Add `nerd_fonts: Option<bool>` to `ConfigFile` in `src/config.rs`, following the same pattern as the existing `log_level` field. Map it through to `AppSettings` as `nerd_fonts: bool` defaulting to `false`.

Users opt in by adding to their `mlbt.toml`:
```toml
nerd_fonts = true
```

---

## Architecture: `Symbols` struct

New file `src/symbols.rs`. A `Symbols` struct is constructed once per frame in `draw::draw()` from `app.settings.nerd_fonts` and passed as a `&'a Symbols` field on each widget struct that needs it — the same pattern as how widgets already carry state references (e.g. `linescore: &'a LineScore`).

No global state. No `Arc`. No `LazyLock`. Just a plain struct passed through the call stack.

```rust
pub struct Symbols {
    nerd_fonts: bool,
}

impl Symbols {
    pub fn new(nerd_fonts: bool) -> Self { Self { nerd_fonts } }

    pub fn selection_cursor(&self) -> char      // '>' or ''
    pub fn scoring_play(&self) -> char          // '!' or ''
    pub fn base_occupied(&self) -> char         // '■' or '◆'
    pub fn base_empty(&self) -> char            // '□' or '◇'
    pub fn scroll_up(&self) -> &'static str     // "↑" or ""
    pub fn scroll_down(&self) -> &'static str   // "↓" or ""
    pub fn sort_asc(&self) -> &'static str      // "^" or "↑"
    pub fn sort_desc(&self) -> &'static str     // "v" or "↓"
    pub fn favorite_marker(&self) -> &'static str  // "  " or "★ "
    pub fn tab_icon(&self, tab: MenuItem) -> &'static str
    //   Scoreboard: "" or " "
    //   Gameday:    "" or " "
    //   Stats:      "" or " "
    //   Standings:  "" or "󰓊 "
}
```

All Nerd Font codepoints are from Font Awesome or Material Design Icons — both present in every standard Nerd Font distribution.

---

## Commit 1 — Infrastructure

**Files:** `src/config.rs`, `src/state/app_settings.rs`, `src/symbols.rs`

- Add `nerd_fonts: Option<bool>` to `ConfigFile`; deserialize with `#[serde(default)]`
- Add `nerd_fonts: bool` to `AppSettings`; propagate from `ConfigFile::into()`
- Add `src/symbols.rs` with the `Symbols` struct

No widget changes in this commit. The new field exists but nothing reads it yet. CI passes.

---

## Commit 2 — Nerd Font glyph swaps

**Files:** `src/draw.rs`, `src/ui/schedule.rs`, `src/ui/standings.rs`, `src/ui/stats.rs`, `src/ui/boxscore.rs`, `src/components/game/matchup.rs`, `src/components/game/pitch_event.rs`, `src/ui/gameday/gameday_widget.rs`, `src/ui/gameday/matchup.rs`, `src/ui/gameday/at_bat.rs`, `src/ui/gameday/plays.rs`

Replace hardcoded symbol constants/chars with `Symbols` method calls:

| Location | Current | With Nerd Fonts |
|---|---|---|
| Tab bar (`draw.rs`) | `"Scoreboard"` | `" Scoreboard"` |
| Selection cursor (`schedule.rs`, `standings.rs`) | `'>'` / `SELECTION_SYMBOL` | `''` |
| Scoring play (`plays.rs`, `pitch_event.rs`) | `'!'` / `SCORING_SYMBOL` | `''` |
| Base runners (`matchup.rs`) | `'■'` / `'□'` | `'◆'` / `'◇'` |
| Scroll indicators (`boxscore.rs`) | `"↑"` / `"↓"` | `""` / `""` |
| Sort arrows (`stats.rs`) | `"^"` / `"v"` | `"↑"` / `"↓"` |
| Favorite marker (`schedule.rs`) | implicit sort-to-top | `"★ "` prefix |

`Runners::generate_lines()` and `Matchup::format_scoreboard_lines()` gain a `symbols: &Symbols` parameter. `PitchEvent::as_lines()` gains `&Symbols`. `SCORING_SYMBOL` and `SELECTION_SYMBOL` consts in `plays.rs` are removed in favor of `Symbols` method calls. `Order::arrow_symbol()` in `stats.rs` takes `&Symbols`.

Widget structs that gain `symbols: &'a Symbols`:
- `ScheduleWidget`
- `StandingsWidget`
- `StatsWidget`
- `TeamBatterBoxscoreWidget`
- `InningPlaysWidget`
- `MatchupWidget`
- `AtBatWidget`
- `GamedayWidget` (entry point; passes down to all child widgets above)

---

## Commit 3 — Team color theming

**Files:** `src/components/team_colors.rs` *(new)*, `src/ui/schedule.rs`, `src/ui/standings.rs`

New `src/components/team_colors.rs` with a `LazyLock<HashMap<&'static str, Color>>` keyed by team abbreviation (matching the abbreviations already in `TEAM_IDS`). All 30 current MLB teams defined using each team's primary color as an `ratatui::style::Color::Rgb(r, g, b)`.

When `nerd_fonts = true`, team abbreviations in the scoreboard and team names in standings are wrapped in a `Span::styled` using the team's color. Falls back to unstyled `Span::raw` when the team isn't found in the map (e.g. historical teams) or when `nerd_fonts = false`.

Favorite team star marker (from Commit 2) also receives the team's color when `nerd_fonts = true`.

---

## Commit 4 — Hit-type labels & count coloring

**Files:** `src/ui/gameday/plays.rs`, `src/components/game/matchup.rs`

> **Note for PR:** These changes are UX improvements gated under `nerd_fonts` for bundling convenience. They do not require a Nerd Font to render correctly. The maintainer may choose to split these into a separate feature or enable them unconditionally.

### Hit-type labels (`plays.rs`)

Replace the `!` / `-` at-bat prefix in `format_runs()` with a fixed-width 3-character event label derived from the play's event codes and count:

| Outcome | Label | Color |
|---|---|---|
| Home run | `HR ` | Blue |
| Triple | `3B ` | Blue |
| Double | `2B ` | Blue |
| Single | `1B ` | Blue |
| Walk / HBP | ` BB` | Green |
| Strikeout | `  K` | Red |
| Other out | `OUT` | Red dim |
| In progress | `...` | White dim |
| Scoring non-hit | `` (icon) | Blue |

The label is always 3 chars wide so play descriptions stay left-aligned.

### Count coloring (`matchup.rs`)

Replace the monochrome `● ◯ ◯` outs string with individually-styled `Span`s: filled dots in red for recorded outs, empty dots dimmed. Balls are green filled/dimmed empty; strikes are red. This requires splitting the existing single-string match arms into `Line`s composed of multiple `Span`s.

---

## Testing

Existing tests in `api/` use `mockito` and are unaffected — no API changes.

Unit tests to add in `src/symbols.rs`:
- `Symbols::new(false)` returns plain ASCII for every method
- `Symbols::new(true)` returns Nerd Font chars for every method

Unit tests to add in `src/components/team_colors.rs`:
- All 30 current teams resolve to a `Some(Color)`
- Unknown abbreviation returns `None`

No snapshot or integration tests — consistent with what the project already has.

---

## Deferred: Additional Team Color Opportunities

These locations display team names/abbreviations but have not yet been colored. Documented for future work.

### Probable Pitchers team column (`src/ui/probable_pitchers.rs`)

`ProbablePitchersWidget` renders a two-row table (away pitcher / home pitcher) where the first column is the team name or abbreviation depending on terminal width. Currently no `symbols` field on the widget.

**Work needed:**
- Add `symbols: &'a Symbols` to `ProbablePitchersWidget`
- Thread it in from the two call sites: `draw.rs` (`draw_scoreboard`) and wherever else the widget is constructed
- In `render()`, build a styled `Cell` for the team column using `team_colors::get(abbreviation, false)` when `symbols.team_colors()` is true
- `ProbablePitcher::to_row_cells()` returns `Vec<Cell>` — either pass `symbols` to it or handle the first cell in the widget's `render()` method directly

### Scoring summary span in play-by-play (`src/ui/gameday/plays.rs`)

`build_scoring_span()` produces a single `Span<'static>` formatted as `[ATH 2, NYY 3]` appended after scoring plays. Currently bold, no team color.

Coloring each abbreviation separately requires splitting it into a `Line` with multiple spans. This is a larger refactor because:
- The function returns `Span<'static>` and callers expect a single span
- The abbreviations are `&'static str` but passing `symbols` requires a lifetime-aware signature change
- The play line is already visually busy with hit-type labels (commit 4)

**Work needed:**
- Change `build_scoring_span` return type from `Span<'static>` to `Line<'static>`
- Update all callers that push this into a `Line::from(vec![...])` to use `extend` or flatten
- Add `symbols: &Symbols` parameter and apply `team_colors::get` to each abbreviation span

---

## What is not changing

- No changes to the `mlb-api` crate
- No changes to the network layer, state management, or key bindings
- No new dependencies
- `nerd_fonts = false` (the default) produces zero diff in rendered output vs. the current codebase
