use crate::components::game::player::Player;
use crate::state::app_state::HomeOrAway;
use mlb_api::boxscore::{LabelValue, Player as ApiPlayer, Team};
use mlb_api::live::LiveResponse;
use std::collections::HashMap;
use tui::prelude::{Line, Stylize};
use tui::style::Color;
use tui::text::Span;
use tui::widgets::Cell;

const SECONDARY_COLOR: Color = Color::DarkGray;

#[derive(Default)]
pub struct Boxscore {
    home_batting: Vec<BatterBoxscore>,
    home_pitching: Vec<PitcherBoxscore>,
    home_batting_notes: Vec<Note>,
    away_batting: Vec<BatterBoxscore>,
    away_pitching: Vec<PitcherBoxscore>,
    away_batting_notes: Vec<Note>,
    game_notes: Vec<GameNote>,
}

#[derive(Default)]
pub struct BatterBoxscore {
    name: String,
    position: String,
    at_bats: u16,
    runs: u16,
    hits: u16,
    rbis: u16,
    walks: u16,
    strike_outs: u16,
    left_on: u16,
    batting_average: String,
    note: Option<String>,
    is_substitute: bool,
}

#[derive(Default)]
pub struct PitcherBoxscore {
    name: String,
    innings_pitched: String,
    hits: u8,
    runs: u8,
    earned_runs: u8,
    walks: u8,
    strikeouts: u8,
    home_runs: u8,
    era: String,
    #[allow(dead_code)]
    pitches: u8,
    #[allow(dead_code)]
    strikes: u8,
    note: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Note {
    Batting(BattingNote),
    Game(GameNote),
}

#[derive(Debug, Default, Clone)]
pub struct BattingNote {
    label: String,
    value: String,
}

#[derive(Debug, Default, Clone)]
pub struct GameNote {
    label: String,
    value: String,
}

impl BatterBoxscore {
    pub fn from_data(
        player: &ApiPlayer,
        player_name: &Player,
        note: Option<String>,
        is_substitute: bool,
    ) -> Self {
        BatterBoxscore {
            name: player_name.boxscore_name.clone(),
            position: player.position.abbreviation.to_string(),
            at_bats: player.stats.batting.at_bats.unwrap_or(0),
            runs: player.stats.batting.runs.unwrap_or(0),
            hits: player.stats.batting.hits.unwrap_or(0),
            rbis: player.stats.batting.rbi.unwrap_or(0),
            walks: player.stats.batting.base_on_balls.unwrap_or(0),
            strike_outs: player.stats.batting.strike_outs.unwrap_or(0),
            left_on: player.stats.batting.left_on_base.unwrap_or(0),
            batting_average: player
                .season_stats
                .batting
                .avg
                .clone()
                .unwrap_or_else(|| "---".to_string()),
            note,
            is_substitute,
        }
    }

    pub fn to_cells(&self) -> Vec<Cell> {
        let note = self.note.as_deref().unwrap_or_default();
        let prefix = match self.is_substitute {
            true => "  ".to_string(),
            false => "".to_string(),
        };
        let name = if self.name == "Totals" {
            Span::from("Totals").fg(SECONDARY_COLOR).into()
        } else {
            Line::from(vec![
                Span::from(format!("{prefix}{note}{} ", self.name)),
                Span::from(self.position.clone()).fg(SECONDARY_COLOR),
            ])
        };

        vec![
            Cell::from(name),
            Cell::from(self.at_bats.to_string()),
            Cell::from(self.runs.to_string()),
            Cell::from(self.hits.to_string()),
            Cell::from(self.rbis.to_string()),
            Cell::from(self.walks.to_string()),
            Cell::from(self.strike_outs.to_string()),
            Cell::from(self.left_on.to_string()),
            Cell::from(self.batting_average.to_string()),
        ]
    }
}

impl PitcherBoxscore {
    pub fn from_data(player: &ApiPlayer, player_name: &Player, note: Option<String>) -> Self {
        PitcherBoxscore {
            name: player_name.boxscore_name.clone(),
            innings_pitched: player
                .stats
                .pitching
                .innings_pitched
                .clone()
                .unwrap_or_else(|| "0".to_string()),
            hits: player.stats.pitching.hits.unwrap_or(0) as u8,
            runs: player.stats.pitching.runs.unwrap_or(0) as u8,
            earned_runs: player.stats.pitching.earned_runs.unwrap_or(0) as u8,
            walks: player.stats.pitching.base_on_balls.unwrap_or(0) as u8,
            strikeouts: player.stats.pitching.strike_outs.unwrap_or(0) as u8,
            home_runs: player.stats.pitching.home_runs.unwrap_or(0) as u8,
            era: player
                .season_stats
                .pitching
                .era
                .clone()
                .unwrap_or_else(|| "---".to_string()),
            pitches: player
                .stats
                .pitching
                .pitches_thrown
                .or(player.stats.pitching.number_of_pitches)
                .unwrap_or(0) as u8,
            strikes: player.stats.pitching.strikes.unwrap_or(0) as u8,
            note,
        }
    }

    pub fn to_cells(&self) -> Vec<Cell> {
        let note = self.note.as_deref().unwrap_or_default();
        let name = if self.name == "Totals" {
            Span::from("Totals").fg(SECONDARY_COLOR).into()
        } else if !note.is_empty() {
            Line::from(vec![
                Span::from(format!("{} ", self.name)),
                Span::from(note).fg(SECONDARY_COLOR),
            ])
        } else {
            self.name.clone().into()
        };

        vec![
            Cell::from(name),
            Cell::from(self.innings_pitched.clone()),
            Cell::from(self.hits.to_string()),
            Cell::from(self.runs.to_string()),
            Cell::from(self.earned_runs.to_string()),
            Cell::from(self.walks.to_string()),
            Cell::from(self.strikeouts.to_string()),
            Cell::from(self.home_runs.to_string()),
            Cell::from(self.era.clone()),
        ]
    }
}

impl Boxscore {
    const NOTE_FILTER: &'static [&'static str] = &[
        "2B",
        "3B",
        "HR",
        "RBI",
        "TB",
        "2-out RBI",
        "Team RISP",
        "E",
        "SB",
    ];
    const BATTING_HEADER: &'static [&'static str] =
        &["player", "ab", "r", "h", "rbi", "bb", "k", "lob", "avg"];

    const PITCHING_HEADER: &'static [&'static str] =
        &["pitcher", "ip", "h", "r", "er", "bb", "k", "hr", "era"];

    const BLANK_LINE_SENTINEL: &'static str = "****";

    pub fn from_live_data(live_game: &LiveResponse, players: &HashMap<u64, Player>) -> Self {
        let (home, away) = match &live_game.live_data.boxscore.teams {
            Some(t) => (&t.home, &t.away),
            None => return Boxscore::default(),
        };
        let home_batting = Boxscore::generate_batting(home, players);
        let home_pitching = Boxscore::generate_pitching(home, players);
        let home_batting_notes = Boxscore::generate_batting_notes(home);

        let away_batting = Boxscore::generate_batting(away, players);
        let away_pitching = Boxscore::generate_pitching(away, players);
        let away_batting_notes = Boxscore::generate_batting_notes(away);

        let game_notes = Boxscore::generate_game_notes(live_game);

        Boxscore {
            home_batting,
            home_pitching,
            home_batting_notes,
            away_batting,
            away_pitching,
            away_batting_notes,
            game_notes,
        }
    }

    fn generate_batting_notes(team: &Team) -> Vec<Note> {
        // at bat notes that correlate to the boxscore, e.g. "a-Walked for Bruján in the 7th."
        let mut ab_notes: Vec<Note> = team
            .note
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|n| n.value.is_some())
            .map(|n| BattingNote::from(n).into())
            .collect();

        // blank line
        if !ab_notes.is_empty() {
            ab_notes.push(
                BattingNote {
                    label: "".to_string(),
                    value: Self::BLANK_LINE_SENTINEL.to_string(),
                }
                .into(),
            );
        }

        // offense notes, e.g. "HR Suzuki 2 (20, 1st inning...)"
        let batting_notes: Vec<Note> = team
            .info
            .iter()
            .flatten()
            .flat_map(|i| &i.field_list)
            // filter out some notes to keep the display smaller
            .filter(|f| f.value.is_some() && Self::NOTE_FILTER.contains(&f.label.as_str()))
            .map(|f| GameNote::from(f).into())
            .collect();

        ab_notes.extend(batting_notes);

        ab_notes
    }

    fn generate_pitching(team: &Team, players: &HashMap<u64, Player>) -> Vec<PitcherBoxscore> {
        let mut pitchers = Vec::new();

        for &player_id in &team.pitchers {
            let player_key = format!("ID{player_id}");
            if let Some(player) = team.players.get(&player_key) {
                if let Some(player_name) = players.get(&player_id) {
                    let note = player.stats.pitching.note.clone();
                    let pitcher = PitcherBoxscore::from_data(player, player_name, note);
                    pitchers.push(pitcher);
                }
            }
        }

        // add total row
        pitchers.push(PitcherBoxscore {
            name: "Totals".to_string(),
            innings_pitched: team
                .team_stats
                .pitching
                .innings_pitched
                .clone()
                .unwrap_or_default(),
            hits: team.team_stats.pitching.hits.unwrap_or_default() as u8,
            runs: team.team_stats.pitching.runs.unwrap_or_default() as u8,
            earned_runs: team.team_stats.pitching.earned_runs.unwrap_or_default() as u8,
            walks: team.team_stats.pitching.base_on_balls.unwrap_or_default() as u8,
            strikeouts: team.team_stats.pitching.strike_outs.unwrap_or_default() as u8,
            home_runs: team.team_stats.pitching.home_runs.unwrap_or_default() as u8,
            era: "".to_string(),
            pitches: 0,
            strikes: 0,
            note: None,
        });

        pitchers
    }

    fn generate_batting(team: &Team, players: &HashMap<u64, Player>) -> Vec<BatterBoxscore> {
        let mut batters = Vec::new();

        for &player_id in &team.batters {
            let player_key = format!("ID{player_id}");
            if let Some(player) = team.players.get(&player_key) {
                if let Some(batting_order) = &player.batting_order {
                    if let Some(player_name) = players.get(&player_id) {
                        // determine if this is a starter or substitute based on batting order
                        let is_starter = batting_order.ends_with('0');
                        let batter = BatterBoxscore::from_data(
                            player,
                            player_name,
                            player.stats.batting.note.clone(),
                            !is_starter,
                        );
                        batters.push(batter);
                    }
                }
            }
        }

        // add total row
        batters.push(BatterBoxscore {
            name: "Totals".to_string(),
            position: "".to_string(),
            at_bats: team.team_stats.batting.at_bats.unwrap_or_default(),
            runs: team.team_stats.batting.runs.unwrap_or_default(),
            hits: team.team_stats.batting.hits.unwrap_or_default(),
            rbis: team.team_stats.batting.rbi.unwrap_or_default(),
            walks: team.team_stats.batting.base_on_balls.unwrap_or_default(),
            strike_outs: team.team_stats.batting.strike_outs.unwrap_or_default(),
            left_on: team.team_stats.batting.left_on_base.unwrap_or_default(),
            batting_average: "".to_string(),
            note: None,
            is_substitute: false,
        });

        batters
    }

    pub fn to_batting_table_rows(&self, active: HomeOrAway) -> Vec<Vec<Cell>> {
        match active {
            HomeOrAway::Home => self.home_batting.iter().map(|p| p.to_cells()).collect(),
            HomeOrAway::Away => self.away_batting.iter().map(|p| p.to_cells()).collect(),
        }
    }

    pub fn get_batting_header(&self) -> &'static [&'static str] {
        Self::BATTING_HEADER
    }

    pub fn to_pitching_table_rows(&self, active: HomeOrAway) -> Vec<Vec<Cell>> {
        match active {
            HomeOrAway::Home => self.home_pitching.iter().map(|p| p.to_cells()).collect(),
            HomeOrAway::Away => self.away_pitching.iter().map(|p| p.to_cells()).collect(),
        }
    }

    pub fn get_pitching_header(&self) -> &'static [&'static str] {
        Self::PITCHING_HEADER
    }

    pub fn get_batting_notes(&self, active: HomeOrAway) -> &[Note] {
        match active {
            HomeOrAway::Home => &self.home_batting_notes,
            HomeOrAway::Away => &self.away_batting_notes,
        }
    }

    pub fn get_game_notes(&self) -> Vec<Line> {
        self.game_notes.iter().filter_map(|n| n.to_line()).collect()
    }

    fn generate_game_notes(live_response: &LiveResponse) -> Vec<GameNote> {
        live_response
            .live_data
            .boxscore
            .info
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|i| i.value.is_some())
            .map(GameNote::from)
            .collect()
    }
}

impl From<GameNote> for Note {
    fn from(note: GameNote) -> Self {
        Note::Game(note)
    }
}
impl From<BattingNote> for Note {
    fn from(note: BattingNote) -> Self {
        Note::Batting(note)
    }
}

impl Note {
    pub fn to_line<'a>(&self) -> Option<Line<'a>> {
        match self {
            Note::Batting(n) => n.to_line(),
            Note::Game(n) => n.to_line(),
        }
    }
}

impl GameNote {
    pub fn to_line<'a>(&self) -> Option<Line<'a>> {
        match (self.label.is_empty(), self.value.is_empty()) {
            (false, false) => Some(Line::from(vec![
                Span::from(format!("{}: ", self.label)).bold(),
                Span::from(self.value.clone()),
            ])),
            (_, _) => None,
        }
    }
}

impl From<&LabelValue> for GameNote {
    fn from(value: &LabelValue) -> Self {
        Self {
            label: value.label.clone(),
            value: value.value.clone().unwrap_or_default(),
        }
    }
}

impl BattingNote {
    pub fn to_line<'a>(&self) -> Option<Line<'a>> {
        match (self.label.is_empty(), self.value.is_empty()) {
            (false, false) => Some(Line::from(format!("{}-{}", self.label, self.value))),
            (true, false) if self.value.as_str() == Boxscore::BLANK_LINE_SENTINEL => {
                Some(Line::default())
            }
            (_, _) => None,
        }
    }
}

impl From<&LabelValue> for BattingNote {
    fn from(value: &LabelValue) -> Self {
        Self {
            label: value.label.clone(),
            value: value.value.clone().unwrap_or_default(),
        }
    }
}
