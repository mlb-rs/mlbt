# mlbt

[![CI](https://github.com/mlb-rs/mlbt/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/mlb-rs/mlbt/actions/workflows/ci.yml)
[![dependency status](https://deps.rs/repo/github/mlb-rs/mlbt/status.svg)](https://deps.rs/repo/github/mlb-rs/mlbt)
[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)

mlb.com in your terminal. Gameday, scores, stats, standings, teams, and player
profiles. Powered by MLB's Stats API, check today's games or dig through decades
of historical information. Go beyond the broadcast and nerd out with win
probability, leverage index, exit velo, and more.

This fork adds optional visual enhancements — color themes, Nerd Font icons,
team colors, and weather display. All additions are off by default; without
any config changes the app behaves identically to upstream.

Color and glyphs serve the same purpose: making dense data scannable. Color
highlights stat tiers at a glance; glyphs provide a redundant signal that
works on low-contrast displays or for users who can't distinguish colors.
Neither is the only channel — text labels are always present.

<img src="https://github.com/user-attachments/assets/1c11e22b-df11-46df-8774-5783b77def84" alt="Demo showing the Schedule, Gameday, Stats, and Standings."/>

## Table of Contents

- [Installation](#installation)
    - [Cargo](#cargo)
    - [Homebrew](#homebrew)
    - [Binaries](#binaries)
    - [Docker](#docker)
- [Features](#features)
- [Usage](#usage)
    - [Scoreboard](#scoreboard)
    - [Gameday](#gameday)
    - [Stats](#stats)
    - [Standings](#standings)
    - [Team Page](#team-page)
    - [Player Profile](#player-profile)
    - [Date Picker](#date-picker)
    - [Help](#help)
- [Config](#config)
- [Shout out](#shout-out)
- [Copyright Notice](#copyright-notice)
- [License](#license)

## Installation

### Cargo

Install a pre-built binary using [cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```bash
cargo binstall mlbt
```

Or build from source:

```bash
cargo install mlbt
```

### Homebrew

```bash
brew install mlb-rs/mlbt/mlbt
```

To update to the latest version:

```bash
brew upgrade mlbt
```

### Binaries

macOS, Linux, and Windows binaries are available on the
[releases](https://github.com/mlb-rs/mlbt/releases) page.

| Platform               | Target                          |
|------------------------|---------------------------------|
| macOS (Apple Silicon)  | `aarch64-apple-darwin`          |
| macOS (Intel)          | `x86_64-apple-darwin`           |
| Linux (x86_64)         | `x86_64-unknown-linux-gnu`      |
| Linux (x86_64, static) | `x86_64-unknown-linux-musl`     |
| Linux (ARM64)          | `aarch64-unknown-linux-gnu`     |
| Linux (ARM64, static)  | `aarch64-unknown-linux-musl`    |
| Linux (ARMv7)          | `armv7-unknown-linux-gnueabihf` |
| Windows (x86_64)       | `x86_64-pc-windows-msvc`        |

`.deb` and `.rpm` packages are also available for Linux x86_64 and ARM64.

### Docker

`mlbt` publishes docker images on [ghcr](https://github.com/mlb-rs/mlbt/pkgs/container/mlbt).

```bash
docker run -it --rm --name mlbt ghcr.io/mlb-rs/mlbt
```

`mlbt` follows [semver](https://semver.org/) practices.
You can execute individual releases explicitly.

```bash
docker run -it --rm --name mlbt ghcr.io/mlb-rs/mlbt:v0.2.0
```

Alternatively build the `mlbt` image with:

```bash
docker build -t mlbt .
```

Execute `mlbt` within the container with:

```bash
docker run -it --rm --name mlbt mlbt:latest
```

## Features

- scoreboard and box score
    - sorted by favorite team
    - full box score
    - probable pitchers for upcoming games
    - win probability graph
    - weather conditions for live/completed games
    - selectable date

- gameday
    - pitch display
    - batter strike zone with heat map coloring
    - selectable at bats (view the pitches and outcome of any at bat in the game)
    - hit stats: exit velocity, launch angle, distance
    - ABS challenge information for 2026+ games
    - leverage index and win probability change per at bat
    - weather conditions with Nerd Font icons

- pitching and hitting stats
    - player stats
    - team stats
    - Fangraphs-inspired stat coloring (ERA, AVG, win%)
    - sorting
    - fuzzy search for players and teams
    - selectable date

- standings
    - sorted by favorite team
    - division/league view
    - stat coloring (win%, streak, run differential)
    - selectable date

- team page
    - roster (active and 40-man)
    - schedule with calendar view
    - recent transactions

- player profile
    - player bio
    - career stats
    - recent games

- visual customization
    - three color themes: lean, classic, rainbow
    - optional Nerd Font icons (tab icons, weather, base runners, play labels)
    - optional team colors on names
    - favorite team highlighting
    - configuration via TOML file

## Usage

After installing, run `mlbt` from your terminal to open the program.

Press `q` to exit at any time.

### Tabs

There are four main tabs.

- Scoreboard
- Gameday
- Stats
- Standings

Press `f` for full screen mode to hide the tab bar.

### Scoreboard

Press `1` to activate this tab.

| Key                 | Description                                            |
|---------------------|--------------------------------------------------------|
| `j` / `↓`           | move down                                              |
| `k` / `↑`           | move up                                                |
| `Enter`             | view current game in Gameday                           |
| `:`                 | activate date picker (see [Date Picker](#date-picker)) |
| `w`                 | toggle win probability graph                           |
| `h`                 | switch to home team in box score                       |
| `a`                 | switch to away team in box score                       |
| `Shift` +  `j`/ `↓` | scroll box score down                                  |
| `Shift` +  `k`/ `↑` | scroll box score up                                    |

### Gameday

Press `2` to activate this tab.

By default, the `info` and `pitches` panes are shown. However, each pane can be
toggled on and off using:

| Key | Description                  |
|-----|------------------------------|
| `i` | info pane                    |
| `p` | pitches pane                 |
| `b` | box score pane               |
| `w` | toggle win probability graph |

To view different at bats in the game, use:

| Key       | Description                                    |
|-----------|------------------------------------------------|
| `j` / `↓` | move to previous at bat                        |
| `k` / `↑` | move to next at bat                            |
| `l`       | move to the "live" at bat, or latest available |
| `s`       | move to first at bat of the game               |

To interact with the box score, use:

| Key                 | Description                      |
|---------------------|----------------------------------|
| `h`                 | switch to home team in box score |
| `a`                 | switch to away team in box score |
| `Shift` +  `j`/ `↓` | scroll box score down            |
| `Shift` +  `k`/ `↑` | scroll box score up              |

### Stats

Press `3` to activate this tab.

Inside the stats tab there are two panes: *stats table* and *options*. The stats
table is used for selecting players/teams and searching. The options pane is
used for sorting the stats and toggling columns on/off.

| Key                 | Description                                            |
|---------------------|--------------------------------------------------------|
| `←` / `→` / `Tab`   | switch between stats table and options pane            |
| `j` / `↓`           | move down in active pane                               |
| `k` / `↑`           | move up in active pane                                 |
| `Shift` + `j` / `↓` | page down in stats table                               |
| `Shift` + `k` / `↑` | page up in stats table                                 |
| `:`                 | activate date picker (see [Date Picker](#date-picker)) |

You can switch between `pitching` and `hitting` stats and filter based on `team`
or `player` using:

| Key | Description |
|-----|-------------|
| `p` | pitching    |
| `h` | hitting     |
| `t` | team        |
| `l` | player      |

#### Search

You can fuzzy search for a player or team in the stats table using:

| Key          | Description                |
|--------------|----------------------------|
| `Ctrl` + `f` | activate fuzzy search      |
| `Enter`      | finish fuzzy search        |
| `Esc`        | clear fuzzy search results |

While the stats table is active, press `Enter` to view a
[player profile](#player-profile) or a [team page](#team-page).

#### Stats Options

The stats options pane can be turned on/off with `o`.

Within each stat group (pitching or hitting) you can toggle the display of
individual stat columns by selecting the stat with `Enter`. To sort the stats by
a column, you can press `s`. To flip the sort order from ascending to descending
or vice versa press `s` again.

| Key     | Description                           |
|---------|---------------------------------------|
| `Enter` | toggle stat column                    |
| `s`     | sort by the currently selected column |
| `o`     | toggle options pane                   |

> If your terminal is too small to display all columns, they will be turned off
> starting from the right side.

### Standings

Press `4` to activate this tab.

| Key       | Description                                            |
|-----------|--------------------------------------------------------|
| `j` / `↓` | move down                                              |
| `k` / `↑` | move up                                                |
| `Enter`   | view [team page](#team-page)                           |
| `:`       | activate date picker (see [Date Picker](#date-picker)) |
| `l`       | toggle division/league view                            |

### Player Profile

The player profile shows a player's career stats and recent games. It can be
opened from [Stats](#stats) or from a [team page](#team-page) roster by pressing
`Enter`.

| Key                 | Description          |
|---------------------|----------------------|
| `s`                 | toggle stat category |
| `j` / `↓`           | scroll down          |
| `k` / `↑`           | scroll up            |
| `Shift` + `j` / `↓` | page down            |
| `Shift` + `k` / `↑` | page up              |
| `Esc`               | close profile        |

### Team Page

The team page shows a team's roster, schedule, and recent transactions. It can
be opened from [Standings](#standings) or from [Stats](#stats) by pressing
`Enter`.

| Key                 | Description                                        |
|---------------------|----------------------------------------------------|
| `←` / `→` / `Tab`   | switch section                                     |
| `j` / `↓`           | move down                                          |
| `k` / `↑`           | move up                                            |
| `Shift` + `j` / `↓` | page down                                          |
| `Shift` + `k` / `↑` | page up                                            |
| `c`                 | toggle calendar                                    |
| `r`                 | toggle roster type                                 |
| `Enter`             | view [player profile](#player-profile) from roster |
| `Esc`               | close team page                                    |

### Date Picker

With the date picker active, input a date in the form of `YYYY-MM-DD`, or use
the `left`/`right` arrow keys, and press `Enter`.

| Key           | Description                     |
|---------------|---------------------------------|
| `←` / `→`     | use arrow keys to navigate date |
| `Enter`       | confirm the selected date       |
| `Esc`         | cancel selection                |
| `today` / `t` | go back to the current day      |

> Note that each tab has its own date, i.e. if you're viewing older stats or
> standings, the schedule can be the current date.

### Help

Press `?` from any tab to open the help page.

| Key                 | Description    |
|---------------------|----------------|
| `j` / `↓`           | move down      |
| `k` / `↑`           | move up        |
| `Shift` + `j` / `↓` | page down      |
| `Shift` + `k` / `↑` | page up        |
| `Esc`               | close help box |
| `"`                 | display logs   |

## Config

You can configure the TUI with the toml file located at your users' home
directory. For a user named `Alice` this would be:

- Linux:   `/home/alice/.config/mlbt/mlbt.toml`
- Windows: `C:\Users\Alice\AppData\Roaming\mlbt\mlbt.toml`
- macOS:   `/Users/Alice/Library/Application Support/mlbt/mlbt.toml`

> You can see the path for your user in the `Help` page.

### Available settings

| Setting | Description | Default |
|---------|-------------|---------|
| `favorite_team` | Highlight your team in the schedule and standings. Use the full team name (e.g. `"Chicago Cubs"`). See [constants.rs](src/components/constants.rs) for all options. | none |
| `timezone` | Time zone for game start times. Common values: `"US/Pacific"`, `"US/Mountain"`, `"US/Central"`, `"US/Eastern"`. [Full list](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones). | `"US/Pacific"` |
| `nerd_fonts` | Enable [Nerd Font](https://www.nerdfonts.com/) icons for tabs, weather, base runners, and play-by-play labels. Requires a Nerd Font installed in your terminal. | `false` |
| `team_colors` | Color team names in the scoreboard and standings using each team's primary color. | `false` |
| `theme` | Color theme tier. `"lean"` is minimal (stock look), `"classic"` adds warm accents and Fangraphs-style stat colors, `"rainbow"` adds colored stat backgrounds and live game highlights. | `"classic"` |
| `log_level` | Log level: `"off"`, `"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"`. | `"error"` |

### Example config

```toml
favorite_team = "San Francisco Giants"
timezone = "US/Pacific"
nerd_fonts = true
team_colors = true
theme = "classic"
```

## Shout out

This was originally built with the
wonderful [tui-rs](https://github.com/fdehau/tui-rs). It is now using the also
wonderful fork, [ratatui](https://github.com/ratatui/ratatui).

These TUIs were extremely helpful:
[spotify-tui](https://github.com/Rigellute/spotify-tui),
[tickrs](https://github.com/tarkah/tickrs),
[bottom](https://github.com/ClementTsang/bottom).

A reference MLB stats API client by
[toddrob99](https://github.com/toddrob99/MLB-StatsAPI) helped make up for the
lack of API documentation.

## Copyright Notice

The data used in this application is supplied by the MLB's Stats API. Use of
this data is subject to the license posted here:
http://gdx.mlb.com/components/copyright.txt.

This application and its author are not affiliated with the MLB.

## License

This project is under the
[MIT License](https://github.com/mlb-rs/mlbt/blob/main/LICENSE).
