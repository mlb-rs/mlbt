# Changelog

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