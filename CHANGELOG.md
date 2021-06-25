# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Standings view: [PR 19](https://github.com/mlb-rs/mlbt/pull/19)
- Added integration tests for the API
- Updated `Help` display to alert user if terminal is too small: [PR 20](https://github.com/mlb-rs/mlbt/pull/20)
- Added a date picker to view schedule on a different day: [PR 21](https://github.com/mlb-rs/mlbt/pull/21)

## [0.0.7] - 2021-06-05

- Separate threads for network calls and rendering.
- Cleaned up some of the rendering code by using the `StatefulWidget` trait.

## [0.0.6] - 2021-06-02

- Hot fixes for an API error and layout bug

## [0.0.5] - 2021-05-30

- Pitches are displayed in the correct location in the strike zone!
- Dialed up the Gameday view, which added:
  - play information for the inning
  - team box score
  - on deck and in the hole batters
- Changed the layout to a toggle-able three pane style.

## [0.0.4] - 2021-05-20

- Added pitch display (currently in the wrong locations relative to heatmap).
- Added on-base and inning information.
- Both of those required changes to `live` API response.

## [0.0.3] - 2021-05-10

- Added heatmap display for current batter. The size of the heatmap needs to set dynamically still.
- Added some basic debug info - display with the "d" key.
- Refactored the rendering code to be contained in the `src/ui` directory.