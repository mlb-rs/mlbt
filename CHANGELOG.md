# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed

- The Schedule view now displays the team record and the losing team is 
  completely greyed out. [PR 62](https://github.com/mlb-rs/mlbt/pull/62)
- Standings are now sorted by the configured favorite team, so the division with
  the team is always shown first. Additionally, the favorite team is 
  automatically selected and highlighted. [PR 58](https://github.com/mlb-rs/mlbt/pull/58)

### Added

- Add win probability API and graphs. Get an overview of the win probability of
  a game in the Schedule view and a more detailed breakdown in Gameday view.
  Press `w` to toggle the win probability graphs on or off. [PR 61](https://github.com/mlb-rs/mlbt/pull/61)
- Add selection for at bats in Gameday view. Use `j` and `k` to scroll through
  at bats and see all the pitches and events for that at bat. [PR 59](https://github.com/mlb-rs/mlbt/pull/59)
- Add a spinning loader when API calls are in flight: [PR 56](https://github.com/mlb-rs/mlbt/pull/56)

## [0.0.15] - 2025-06-03

### Fixed

- Error in team stats API response parsing introduced by the date selection update

## [0.0.14] - 2025-05-31

### Added

- Add sorting to stats view! Use `s` to sort the stats by a column: [PR 52](https://github.com/mlb-rs/mlbt/pull/52)
- Add date picker to Standings and Stats! [PR 53](https://github.com/mlb-rs/mlbt/pull/53)
- Add `timezone` to the config file. This changes which time zone is used to 
  display the start time of games in the Schedule.

### Changed

- Update GameDay UI to display more information [PR 50](https://github.com/mlb-rs/mlbt/pull/50):
  - play descriptions now wrap lines
  - if runs are scored the new game score is shown
  - other events (e.g. pickoff attempt, wild pitch, mount visit, etc.) that
    happen during the at bat are now shown

## [0.0.13] - 2025-04-02

### Changed

- Update Rust to 1.85 and 2024 version
- Update `ratatui` to 0.29.0

### Fixed

- The Athletics name was showing up as `unknown` because Oakland is no longer in
  the name. RIP
- The standings API had a couple fields that should be optional
- Default to white if a pitch color is missing from the API

## [0.0.12] - 2023-06-25

### Added

- Config file for setting your favorite team, which will always be first in 
  schedule view: [Issue 16](https://github.com/mlb-rs/mlbt/issues/16)
- Change the schedule date with arrow keys: use `left` for the previous day and
  `right` for the next day

### Changed

- Move API to be async and do api calls concurrently when possible: [Issue 13](https://github.com/mlb-rs/mlbt/issues/13)
- Switch from `tui-rs` to a new (maintained) fork `ratatui`. Thanks `tui-rs`!
- Update dependencies and refactor code a bit

### Fixed

- Getting stuck in help menu: [Issue 29](https://github.com/mlb-rs/mlbt/issues/29)

## [0.0.11] - 2022-04-13

### Changed

- Update to Rust 2021 version
- Update dependencies, notably `tui-rs` to 0.17.0

### Fixed

- Crash due to Indian Guardians name change: [Issue 27](https://github.com/mlb-rs/mlbt/issues/27)

## [0.0.10] - 2021-09-23

### Changed

- Update `tui-rs` to 0.16.0, which fixed table flickering
- Update `crossterm` to 0.21 and `chrono_tz` to 0.6

### Fixed

- Table column flickering: [Issue 10](https://github.com/mlb-rs/mlbt/issues/10)
- API error when deserializing season stats, as it turns out batters leave more
  than 256 men on base in a season. Switched everything to `u16`s.

## [0.0.9] - 2021-07-13

### Added

- Stats view: [PR 25](https://github.com/mlb-rs/mlbt/pull/25)
- Added a panic hook to print a nice stack trace on crash

### Fixed

- All Star game caused a crash when the schedule was loaded: [PR 24](https://github.com/mlb-rs/mlbt/pull/24)

## [0.0.8] - 2021-06-29

### Added

- Standings view: [PR 19](https://github.com/mlb-rs/mlbt/pull/19)
- Added integration tests for the API
- Added a date picker to view schedule on a different day: [PR 21](https://github.com/mlb-rs/mlbt/pull/21)

### Changed

- Updated `Help` display to alert user if terminal is too small: [PR 20](https://github.com/mlb-rs/mlbt/pull/20)
- UI tweaks: [PR 22](https://github.com/mlb-rs/mlbt/pull/22)

## [0.0.7] - 2021-06-05

### Changed

- Separate threads for network calls and rendering.
- Cleaned up some of the rendering code by using the `StatefulWidget` trait.

## [0.0.6] - 2021-06-02

### Fixed

- Hot fixes for an API error and layout bug

## [0.0.5] - 2021-05-30

### Changed

- Pitches are displayed in the correct location in the strike zone!
- Dialed up the Gameday view, which added:
  - play information for the inning
  - team box score
  - on deck and in the hole batters
- Changed the layout to a toggle-able three pane style.

## [0.0.4] - 2021-05-20

### Added

- Added pitch display (currently in the wrong locations relative to heatmap).
- Added on-base and inning information.
- Both of those required changes to `live` API response.

## [0.0.3] - 2021-05-10

### Added

- Added heatmap display for current batter. The size of the heatmap needs to set dynamically still.
- Added some basic debug info - display with the "d" key.

### Changed

- Refactored the rendering code to be contained in the `src/ui` directory.
