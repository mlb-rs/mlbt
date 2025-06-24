use crate::components::boxscore::Boxscore;
use crate::components::constants::TEAM_IDS;
use crate::components::game::at_bat::AtBat;
use crate::components::game::player::{Player, PlayerStats};
use crate::components::game::win_probability::WinProbability;
use crate::components::linescore::LineScore;
use crate::components::standings::Team;
use indexmap::IndexMap;
use mlb_api::live::LiveResponse;
use mlb_api::plays::Play;
use mlb_api::win_probability::WinProbabilityResponse;
use std::collections::HashMap;
use std::sync::LazyLock;

pub type AtBatIndex = u8;
pub type PlayerId = u64;
pub type PlayerMap = HashMap<PlayerId, Player>;

static DEFAULT_AT_BAT: LazyLock<AtBat> = LazyLock::new(AtBat::default);

#[derive(Default)]
pub struct GameState {
    pub game_id: u64,
    pub home_team: Team,
    pub away_team: Team,
    pub linescore: LineScore,
    pub boxscore: Boxscore,
    pub current_at_bat: AtBatIndex,
    pub at_bats: IndexMap<AtBatIndex, AtBat>,
    pub on_deck: Option<PlayerId>,
    pub in_hole: Option<PlayerId>,
    pub win_probability: WinProbability,
    pub players: PlayerMap,
}

impl GameState {
    /// Update with latest data from the API.
    pub fn update(&mut self, live_data: &LiveResponse, win_probability: &WinProbabilityResponse) {
        if self.game_id != live_data.game_pk {
            self.reset();
        }
        self.game_id = live_data.game_pk;
        self.players = Self::create_players(live_data); // do this first
        self.set_teams(live_data);
        self.set_on_deck(live_data);
        self.current_at_bat = Self::get_current_play_ab_index(live_data);
        self.boxscore = Boxscore::from_live_data(live_data, &self.players);
        self.linescore = LineScore::from_live_data(live_data);
        if let Some(plays) = &live_data.live_data.plays.all_plays {
            plays.iter().for_each(|p| Self::update_single_play(self, p));
        }
        self.win_probability = WinProbability::from(win_probability);
    }

    fn set_teams(&mut self, live_data: &LiveResponse) {
        self.home_team = TEAM_IDS
            .get(live_data.game_data.teams.home.name.as_str())
            .cloned()
            .unwrap_or_default();
        self.away_team = TEAM_IDS
            .get(live_data.game_data.teams.away.name.as_str())
            .cloned()
            .unwrap_or_default();
    }

    fn set_on_deck(&mut self, live_data: &LiveResponse) {
        self.on_deck = live_data
            .live_data
            .linescore
            .offense
            .on_deck
            .as_ref()
            .map(|od| od.id);
        self.in_hole = live_data
            .live_data
            .linescore
            .offense
            .in_hole
            .as_ref()
            .map(|ih| ih.id);
    }

    fn create_players(live_data: &LiveResponse) -> PlayerMap {
        // get the player names from the game data
        let mut map = HashMap::new();
        for player in live_data.game_data.players.values() {
            map.insert(player.id, Player::from(player));
        }

        // get the player stats from the boxscore
        if let Some(teams) = &live_data.live_data.boxscore.teams {
            for player in teams.home.players.values() {
                if let Some(p) = map.get_mut(&player.person.id) {
                    p.stats = PlayerStats::from(player);
                }
            }
            for player in teams.away.players.values() {
                if let Some(p) = map.get_mut(&player.person.id) {
                    p.stats = PlayerStats::from(player);
                }
            }
        }
        map
    }

    /// Will always return an at bat. If there isn't one, it will return the default.
    pub fn get_latest_at_bat(&self) -> &AtBat {
        self.at_bats
            .get(&self.current_at_bat)
            .unwrap_or_else(|| &DEFAULT_AT_BAT)
    }

    /// May not exist.
    pub fn get_at_bat_by_index(&self, index: u8) -> Option<&AtBat> {
        self.at_bats.get(&index)
    }

    /// Helper function to try to get an at bat by index. If it doesn't exist, it will return the
    /// latest at bat. It will also return `true` if the at bat is the current at bat.
    pub fn get_at_bat_by_index_or_current(&self, index: Option<u8>) -> (&AtBat, bool) {
        let idx = index.unwrap_or(self.current_at_bat);
        match self.get_at_bat_by_index(idx) {
            Some(at_bat) => (at_bat, idx == self.current_at_bat),
            None => (self.get_latest_at_bat(), true),
        }
    }

    pub fn count_events(&self) -> usize {
        self.at_bats.len()
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
        let at_bat = AtBat::from(play);
        self.at_bats.insert(at_bat.index, at_bat);
    }

    pub fn reset(&mut self) {
        *self = Self::default()
    }

    pub fn format_on_deck(&self) -> Option<String> {
        self.on_deck
            .and_then(|id| self.players.get(&id))
            .map(|player| format!("on deck: {}", player.last_name))
    }

    pub fn format_in_hole(&self) -> Option<String> {
        self.in_hole
            .and_then(|id| self.players.get(&id))
            .map(|player| format!("in hole: {}", player.last_name))
    }
}
