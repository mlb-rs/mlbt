# mlbt

[![CI](https://github.com/mlb-rs/mlbt/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/mlb-rs/mlbt/actions/workflows/ci.yml)
[![dependency status](https://deps.rs/repo/github/mlb-rs/mlbt/status.svg)](https://deps.rs/repo/github/mlb-rs/mlbt)
[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)

A terminal user interface for the MLB Statcast API, written in Rust.

Check scores, standings, and stats. Even watch a live game using Gameday!

<img src="https://github.com/user-attachments/assets/1c11e22b-df11-46df-8774-5783b77def84" alt="Demo showing the Schedule, Gameday, Stats, and Standings."/>

## Table of Contents

- [What](#what)
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
    - [Date Picker](#date-picker)
    - [Help](#help)
- [Config](#config)
- [Shout out](#shout-out)
- [Copyright Notice](#copyright-notice)
- [License](#license)

## What

The MLB Statcast API is a publicly available (see its [license](#license)
information below) REST API that you can query to get back almost any
information about a baseball game, past or present. If you've ever watched a
baseball game on TV you've seen the data the API passes around in action. Two
prime examples are the pitch/strike zone overlay, and home run stats (like
distance and launch angle). This is accomplished by MLB's sophisticated Statcast
vision system, which is implemented in every Major League ballpark.

This TUI is an interface for the API, with the intention of providing a light
weight way to consume baseball data. See the [features](#features) listed below
for more details.

A TUI and baseball data make a pretty natural combination, at least compared to
other sports. The Gameday view allows you to "watch" a live game by polling the
API every 10 seconds. This matches the poll rate at which the official Gameday,
found [here](https://www.mlb.com/scores), operates at. The goal with the TUI
version is to mimic the official version as closely as possible.

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
brew tap mlb-rs/mlbt
brew install mlbt
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

`mlbt` [publishes docker images on ghcr](https://github.com/mlb-rs/mlbt/pkgs/container/mlbt).

```bash
docker run -it --rm --name mlbt ghcr.io/mlb-rs/mlbt
```

`mlbt` follows [semver](https://semver.org/) practices.
You can execute individual releases explicitly.

```bash
docker run -it --rm --name mlbt ghcr.io/mlb-rs/mlbt:v0.0.18
```

Alternately build the `mlbt` image with:

```bash
docker build -t mlbt .
```

Execute `mlbt` within the container with:

```bash
docker run -it --rm --name mlbt mlbt:latest
```

## Features

- [X] scoreboard and box score
    - [X] sorted by favorite team
    - [X] selectable date
    - [X] win probability graph

- [X] gameday
    - [X] pitch display
    - [X] batter strike zone with heat map coloring
    - [X] selectable at bats (view the pitches and outcome of any at bat in the
      game)
    - [X] hit stats (exit velocity, launch angle, distance)
    - [X] ABS challenge information for 2026+ games (review status, player that
      challenged)
    - [X] win probability per at bat

- [X] pitching and hitting stats
    - [X] player stats
    - [X] team stats
    - [X] sorting
    - [X] fuzzy search
    - [X] selectable date

- [X] standings
    - [X] sorted by favorite team
    - [X] selectable date
    - [X] division/league view

- [X] configuration
    - [X] favorite team
    - [X] time zone

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

Use `←`/`→`/`Tab` to switch focus between the stats table and the options pane.
When the stats table is focused, `j`/`k` scroll through rows. When the options
pane is focused, `j`/`k` navigate stat columns for toggling and sorting. You can
also use `Shift` + `j`/`k` to page through the stats table.

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

#### Search and Player Profiles

You can fuzzy search for a player or team in the stats pane using:

| Key          | Description                |
|--------------|----------------------------|
| `Ctrl` + `f` | activate fuzzy search      |
| `Enter`      | finish fuzzy search        |
| `Esc`        | clear fuzzy search results |

While the stats table is selected, you can view a player profile for the
currently selected player. Use `s` to toggle the category (e.g. regular season
or spring training). If the player profile doesn't fit on the screen, you can
scroll down and up using `j` and `k`.

> Only players (not teams) can be selected at this time.

| Key       | Description              |
|-----------|--------------------------|
| `Enter`   | open player profile      |
| `Esc`     | clear player profile     |
| `s`       | toggle stat category     |
| `j` / `↓` | scroll down player stats |
| `k` / `↑` | scroll up player stats   |

#### Stats Options

Within each stat group (pitching or hitting) you can toggle the display of
individual stat columns by selecting the stat with `Enter`. This selection pane
can be turned on/off with `o`.

To sort the stats by a column, instead of hitting `Enter` you can press `s`. To
flip the sort order from ascending to descending or vice versa press `s` again.

| Key          | Description                           |
|--------------|---------------------------------------|
| `Enter`      | toggle stat column                    |
| `s`          | sort by the currently selected column |
| `o`          | toggle options pane                   |

> If your terminal is too small to display all columns, they will be turned off
> starting from the right side.

### Standings

Press `4` to activate this tab.

| Key       | Description                                            |
|-----------|--------------------------------------------------------|
| `j` / `↓` | move down                                              |
| `k` / `↑` | move up                                                |
| `:`       | activate date picker (see [Date Picker](#date-picker)) |
| `l`       | toggle division/league view                            |

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

| Key       | Description    |
|-----------|----------------|
| `j` / `↓` | move down      |
| `k` / `↑` | move up        |
| `Esc`     | close help box |
| `"`       | display logs   |

## Config

You can configure the TUI with the toml file located at your users' home
directory. For a user named `Alice` this would be:

- Linux:   `/home/alice/.config/mlbt/mlbt.toml`
- Windows: `C:\Users\Alice\AppData\Roaming\mlbt\mlbt.toml`
- macOS:   `/Users/Alice/Library/Application Support/mlbt/mlbt.toml`

> You can see the path for your user in the `Help` page.

### Available settings

- `favorite_team`: This will make that team always show up first in the schedule
  if they have a game that day.
  See [here](https://github.com/mlb-rs/mlbt/blob/main/src/components/constants.rs#L37)
  for options (note: use the full name and not the short name).
- `timezone`: This will change the time zone of the start time for the games in
  the schedule. The default is `US/Pacific`. Some common options are:
    * `US/Pacific`
    * `US/Mountain`
    * `US/Central`
    * `US/Eastern`
    * For the full list
      see [here](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones)
- `log_level`: Set the log level to be displayed. If not present, `error` level
  is used. Use a lowercase word, e.g. `debug`.
  See [here](https://github.com/mlb-rs/mlbt/blob/main/src/config.rs#L16)
  for the options.

### Example config

```toml
# See https://github.com/mlb-rs/mlbt#config for options
favorite_team = "Chicago Cubs"
timezone = "US/Pacific"
log_levl = "error"
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
