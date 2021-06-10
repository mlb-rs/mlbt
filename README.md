# mlbt

[![CI](https://github.com/andschneider/mlbt/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/andschneider/mlbt/actions/workflows/ci.yml)

A terminal user interface for the MLB Statcast API, written in Rust.

Check scores, standings, and stats. Even watch a live game using Gameday!

> This project is under active development! See the [v0.1.0 Milestone](https://github.com/mlb-rs/mlbt/milestone/1) for more information, planned features, and known bugs.

## Table of Contents

- [What](#what)
- [Installation](#installation)
  - [Homebrew](#homebrew)
  - [Binaries](#binaries)
  - [Cargo](#cargo)
- [Features](#features)
- [Usage](#usage)
  - [Scoreboard](#scoreboard)
  - [Gameday](#gameday)
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

WIP - more platform support

### Homebrew

```bash
brew tap mlb-rs/mlbt
brew install mlbt
```

### Binaries

macOS, Linux, and Windows binaries are available on the
[releases](https://github.com/mlb-rs/mlbt/releases) page.

### Cargo

After cloning or downloading the source:

```bash
cargo install mlbt --path .
```

TODO - add to crates.io

## Features

- [X] scoreboard and box score
  - [ ] selectable date

- [X] gameday

- [ ] standings

- [ ] stats
  - [ ] player stats
  - [ ] team stats
  - [ ] stat search (store in sqlite or an embedded db?)

- [ ] CLI
- [ ] configuration: favorite team, colors, keymap

## Usage

There are four main tabs:

- Scoreboard
- Gameday
- Stats
- Standings

### Scoreboard

Press `1` to activate this tab.

To select different games, use `j` and `k`.

### Gameday

Press `2` to activate this tab.

Each pane can be toggled on and off using:

- info pane: `i`
- pitches pane: `p`
- box score pane: `b`

To switch the team displayed in the box score:

- `h` for the home team
- `a` for the away team

### Stats

Press `3` to activate this tab.

TODO

### Standings

Press `4` to activate this tab.

TODO

### Help

To display a help message with all controls, press `?`. Press `Esc` to close it.

## Config

TODO

## Shout out

This library is built with the wonderful
[tui-rs](https://github.com/fdehau/tui-rs).

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
