use crate::components::boxscore::TeamBatterBoxscore;
use crate::components::game::at_bat::AtBatV2;
use crate::components::game::matchup::Summary;
use crate::components::linescore::LineScore;
use indexmap::IndexMap;
use mlb_api::live::LiveResponse;
use mlb_api::plays::Play;
use std::sync::LazyLock;

type AtBatIndex = u8;

static DEFAULT_AT_BAT: LazyLock<AtBatV2> = LazyLock::new(AtBatV2::default);

#[derive(Default)]
pub struct GameStateV2 {
    pub game_id: u64,
    pub summary: Summary,
    pub linescore: LineScore,
    pub boxscore: TeamBatterBoxscore,
    pub current_at_bat: AtBatIndex,
    pub at_bats: IndexMap<AtBatIndex, AtBatV2>,
    // pub players: HashMap<u64, PlayerStats>, // TODO
}

// pub struct PlayerStats {
// TODO display current ab info, e.g. "1-3, 2B, RBI"
//     pub summary: String,
//     pub note: Option<String>
// }

impl GameStateV2 {
    /// Update with latest data from the API.
    pub fn update(&mut self, live_data: &LiveResponse) {
        if self.game_id != live_data.game_pk {
            self.reset();
        }
        self.game_id = live_data.game_pk;
        self.current_at_bat = Self::get_current_play_ab_index(live_data);
        self.summary = Summary::from(live_data);
        self.boxscore = TeamBatterBoxscore::from_live_data(live_data);
        self.linescore = LineScore::from_live_data(live_data);
        if let Some(plays) = &live_data.live_data.plays.all_plays {
            plays.iter().for_each(|p| Self::update_single_play(self, p));
        }
    }

    pub fn get_summary(&self) -> &Summary {
        &self.summary
    }

    /// Will always return an at bat. If there isn't one, it will return the default.
    pub fn get_latest_at_bat(&self) -> &AtBatV2 {
        self.at_bats
            .get(&self.current_at_bat)
            .unwrap_or_else(|| &DEFAULT_AT_BAT)
    }

    /// May not exist.
    pub fn get_at_bat_by_index(&self, index: u8) -> Option<&AtBatV2> {
        self.at_bats.get(&index)
    }

    /// Helper function to try to get an at bat by index. If it doesn't exist, it will return the
    /// latest at bat. It will also return `true` if the at bat is the current at bat.
    pub fn get_at_bat_by_index_or_current(&self, index: Option<u8>) -> (&AtBatV2, bool) {
        let idx = index.unwrap_or(self.current_at_bat);
        let game = self
            .get_at_bat_by_index(idx)
            .unwrap_or_else(|| self.get_latest_at_bat());
        (game, idx == self.current_at_bat)
    }

    pub fn count_events(&self) -> usize {
        self.at_bats
            .values()
            .map(|ab| ab.play_result.events.len())
            .sum::<usize>()
            + self.at_bats.len()
    }

    fn get_current_play_ab_index(live_data: &LiveResponse) -> AtBatIndex {
        live_data
            .live_data
            .plays
            .current_play
            .as_ref()
            .map(|c| c.about.at_bat_index)
            .unwrap_or(0)
    }

    /// Useful for updating current play.
    pub fn update_single_play(&mut self, play: &Play) {
        let at_bat = AtBatV2::from(play);
        self.at_bats.insert(at_bat.index, at_bat);
    }

    pub fn reset(&mut self) {
        *self = Self::default()
    }
}
