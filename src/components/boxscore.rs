use crate::components::game::player::Player;
use crate::state::app_state::HomeOrAway;
use crate::ui::styling::{DimStyle, avg_style, dim_style, era_style, text_style};
use mlbt_api::boxscore::{LabelValue, Player as ApiPlayer, Team};
use mlbt_api::live::LiveResponse;
use std::collections::HashMap;
use tui::prelude::{Line, Stylize};
use tui::text::Span;
use tui::widgets::Cell;

const TOTALS_NAME: &str = "Totals";

#[derive(Default)]
pub struct Boxscore {
    home_batting: Vec<BatterBoxscore>,
    home_pitching: Vec<PitcherBoxscore>,
    home_batting_notes: Vec<Note>,
    away_batting: Vec<BatterBoxscore>,
    away_pitching: Vec<PitcherBoxscore>,
    away_batting_notes: Vec<Note>,
    game_notes: Vec<Note>,
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

    pub fn to_cells(&self) -> Vec<Cell<'_>> {
        let total = self.name == TOTALS_NAME;

        let name_cell = if total {
            Cell::from(self.name.clone()).dim()
        } else {
            let note = self.note.as_deref().unwrap_or_default();
            let prefix = match self.is_substitute {
                true => "  ",
                false => "",
            };
            Cell::from(Line::from(vec![
                Span::from(format!("{prefix}{note}{} ", self.name)),
                Span::from(self.position.clone()).style(dim_style()),
            ]))
        };

        vec![
            name_cell,
            styled_cell(self.at_bats, self.at_bats.dim_or_default(), total),
            styled_cell(self.runs, self.runs.dim_or_default(), total),
            styled_cell(self.hits, self.hits.dim_or_default(), total),
            styled_cell(self.rbis, self.rbis.dim_or_default(), total),
            styled_cell(self.walks, self.walks.dim_or_default(), total),
            styled_cell(self.strike_outs, self.strike_outs.dim_or_default(), total),
            styled_cell(self.left_on, self.left_on.dim_or_default(), total),
            styled_cell(
                &self.batting_average,
                avg_style(&self.batting_average),
                total,
            ),
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

    pub fn to_cells(&self) -> Vec<Cell<'_>> {
        let total = self.name == TOTALS_NAME;

        let name_cell = if total {
            Cell::from(self.name.clone()).dim()
        } else {
            let note = self.note.as_deref().unwrap_or_default();
            if !note.is_empty() {
                Cell::from(Line::from(vec![
                    Span::from(format!("{} ", self.name)),
                    Span::from(note).style(dim_style()),
                ]))
            } else {
                Cell::from(self.name.clone())
            }
        };

        vec![
            name_cell,
            styled_cell(&self.innings_pitched, text_style(), total),
            styled_cell(self.hits, self.hits.dim_or_default(), total),
            styled_cell(self.runs, self.runs.dim_or_default(), total),
            styled_cell(self.earned_runs, self.earned_runs.dim_or_default(), total),
            styled_cell(self.walks, self.walks.dim_or_default(), total),
            styled_cell(self.strikeouts, self.strikeouts.dim_or_default(), total),
            styled_cell(self.home_runs, self.home_runs.dim_or_default(), total),
            styled_cell(&self.era, era_style(&self.era), total),
        ]
    }
}

impl Boxscore {
    const BLANK_LINE_SENTINEL: &'static str = "****";
    const HEADER_SENTINEL: &'static str = "header";

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
            .flat_map(|i| {
                std::iter::once(
                    GameNote {
                        label: i.title.clone(),
                        value: Self::HEADER_SENTINEL.to_string(),
                    }
                    .into(),
                )
                .chain(i.field_list.iter().map(|f| GameNote::from(f).into()))
            })
            .collect();

        ab_notes.extend(batting_notes);

        ab_notes
    }

    fn generate_pitching(team: &Team, players: &HashMap<u64, Player>) -> Vec<PitcherBoxscore> {
        let mut pitchers = Vec::new();

        for &player_id in &team.pitchers {
            let player_key = format!("ID{player_id}");
            if let Some(player) = team.players.get(&player_key)
                && let Some(player_name) = players.get(&player_id)
            {
                let note = player.stats.pitching.note.clone();
                let pitcher = PitcherBoxscore::from_data(player, player_name, note);
                pitchers.push(pitcher);
            }
        }

        // add total row
        pitchers.push(PitcherBoxscore {
            name: TOTALS_NAME.to_string(),
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
            if let Some(player) = team.players.get(&player_key)
                && let Some(batting_order) = &player.batting_order
                && let Some(player_name) = players.get(&player_id)
            {
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

        // add total row
        batters.push(BatterBoxscore {
            name: TOTALS_NAME.to_string(),
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

    pub fn to_batting_table_rows<'a>(
        &'a self,
        active: HomeOrAway,
    ) -> impl Iterator<Item = Vec<Cell<'a>>> + 'a {
        match active {
            HomeOrAway::Home => self.home_batting.iter().map(BatterBoxscore::to_cells),
            HomeOrAway::Away => self.away_batting.iter().map(BatterBoxscore::to_cells),
        }
    }

    pub fn count_batting_table_rows(&self, active: HomeOrAway) -> usize {
        match active {
            HomeOrAway::Home => self.home_batting.len(),
            HomeOrAway::Away => self.away_batting.len(),
        }
    }

    pub fn to_pitching_table_rows<'a>(
        &'a self,
        active: HomeOrAway,
    ) -> impl Iterator<Item = Vec<Cell<'a>>> + 'a {
        match active {
            HomeOrAway::Home => self.home_pitching.iter().map(PitcherBoxscore::to_cells),
            HomeOrAway::Away => self.away_pitching.iter().map(PitcherBoxscore::to_cells),
        }
    }

    pub fn count_pitching_table_rows(&self, active: HomeOrAway) -> usize {
        match active {
            HomeOrAway::Home => self.home_pitching.len(),
            HomeOrAway::Away => self.away_pitching.len(),
        }
    }

    pub fn get_batting_notes(&self, active: HomeOrAway) -> &[Note] {
        match active {
            HomeOrAway::Home => &self.home_batting_notes,
            HomeOrAway::Away => &self.away_batting_notes,
        }
    }

    pub fn get_game_notes(&self) -> &[Note] {
        &self.game_notes
    }

    fn generate_game_notes(live_response: &LiveResponse) -> Vec<Note> {
        live_response
            .live_data
            .boxscore
            .info
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|i| i.value.is_some())
            .map(|n| GameNote::from(n).into())
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
            (false, false) if self.value == Boxscore::HEADER_SENTINEL => Some(Line::from(vec![
                Span::from(self.label.to_string()).bold().style(dim_style()),
            ])),
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
            (true, false) if self.value == Boxscore::BLANK_LINE_SENTINEL => Some(Line::default()),
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

fn styled_cell<T: ToString, S: Into<tui::style::Style>>(
    val: T,
    style: S,
    is_total: bool,
) -> Cell<'static> {
    let cell = Cell::from(val.to_string());
    if is_total {
        cell.dim()
    } else {
        cell.style(style)
    }
}
