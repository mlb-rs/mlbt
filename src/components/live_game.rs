use crate::components::at_bat::AtBat;
use crate::components::boxscore::TeamBatterBoxscore;
use crate::components::linescore::LineScore;
use crate::components::matchup::{Matchup, MatchupV2, Summary};
use crate::components::plays::{InningPlays, PlayResult};
use mlb_api::live::LiveResponse;
use mlb_api::plays::Play;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Default)]
pub struct GameState {
    pub current_game_id: u64,
    pub linescore: LineScore,
    pub at_bat: AtBat,
    pub boxscore: TeamBatterBoxscore,
    pub matchup: Matchup,
    pub plays: InningPlays,
}

#[derive(Default)]
pub struct GameStateV2 {
    pub game_id: u64,
    pub summary: Summary,
    pub linescore: LineScore,
    pub boxscore: TeamBatterBoxscore,
    pub current_at_bat: u8,
    pub at_bats: HashMap<u8, AtBatV2>,
    // pub players: HashMap<u8, PlayerStats>, // TODO
}

#[derive(Default)]
pub struct AtBatV2 {
    pub index: u8,
    pub inning: u8,
    pub is_top_inning: bool,
    /// Strikezone, pitches, pitch information
    pub pitches: AtBat,
    /// Matchup information
    pub matchup: MatchupV2,
    /// Play information
    pub play_result: PlayResult,
}

static DEFAULT_AT_BAT: LazyLock<AtBatV2> = LazyLock::new(|| AtBatV2::default());

impl GameStateV2 {
    pub fn get_summary(&self) -> &Summary {
        &self.summary
    }

    pub fn get_latest_at_bat(&self) -> &AtBatV2 {
        self.at_bats
            .get(&self.current_at_bat)
            .unwrap_or_else(|| &DEFAULT_AT_BAT)
    }

    /// May not exist.
    pub fn get_at_bat_by_index(&self, index: u8) -> Option<&AtBatV2> {
        self.at_bats.get(&index)
    }

    pub fn update(&mut self, live_data: &LiveResponse) {
        if self.game_id != live_data.game_pk {
            self.reset();
        }
        self.game_id = live_data.game_pk;
        self.current_at_bat = live_data
            .live_data
            .plays
            .current_play
            .as_ref()
            .map(|c| c.about.at_bat_index)
            .unwrap_or(0);
        self.summary = Summary::from(live_data);
        self.boxscore = TeamBatterBoxscore::from_live_data(live_data);
        self.linescore = LineScore::from_live_data(live_data);
        if let Some(plays) = &live_data.live_data.plays.all_plays {
            plays.iter().for_each(|p| Self::update_single_play(self, p));
        }
    }

    /// Useful for updating current play.
    pub fn update_single_play(&mut self, play: &Play) {
        let abv2 = AtBatV2 {
            index: play.about.at_bat_index,
            inning: play.about.inning,
            is_top_inning: play.about.is_top_inning,
            pitches: AtBat::from_play(play),
            matchup: MatchupV2::from_play(play),
            play_result: PlayResult::from_play(play),
        };
        self.at_bats.insert(abv2.index, abv2);
    }

    fn reset(&mut self) {
        self.at_bats.clear();
        self.summary = Summary::default();
        self.current_at_bat = 0;
    }
}

impl GameState {
    pub fn update(&mut self, live_data: &LiveResponse) {
        self.current_game_id = live_data.game_pk;
        self.linescore = LineScore::from_live_data(live_data);
        self.at_bat = AtBat::from_live_data(live_data);
        self.boxscore = TeamBatterBoxscore::from_live_data(live_data);
        self.matchup = Matchup::from_live_data(live_data);
        self.plays = InningPlays::from_live_data(live_data);
    }

    pub fn clear(&mut self) {
        self.current_game_id = 0;
        self.linescore = LineScore::default();
        self.at_bat = AtBat::default();
        self.boxscore = TeamBatterBoxscore::default();
        self.matchup = Matchup::default();
        self.plays = InningPlays::default();
    }

    pub fn get_current_game_id(&self) -> u64 {
        self.current_game_id
    }

    pub fn set_game_id(&mut self, game_id: u64) {
        self.current_game_id = game_id;
    }
}
