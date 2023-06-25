# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

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
