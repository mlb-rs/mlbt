use crate::components::boxscore::TeamBatterBoxscore;
use crate::components::game::at_bat::AtBatV2;
use crate::components::game::matchup::Player;
use crate::components::game::win_probability::WinProbability;
use crate::components::linescore::LineScore;
use crate::components::standings::Team;
use indexmap::IndexMap;
use mlb_api::boxscore::Player as ApiPlayer;
use mlb_api::live::LiveResponse;
use mlb_api::plays::Play;
use mlb_api::win_probability::WinProbabilityResponse;
use std::collections::HashMap;
use std::sync::LazyLock;

pub type AtBatIndex = u8;

static DEFAULT_AT_BAT: LazyLock<AtBatV2> = LazyLock::new(AtBatV2::default);

#[derive(Default)]
pub struct GameStateV2 {
    pub game_id: u64,
    pub home_team: Team,
    pub away_team: Team,
    // pub summary: Summary,
    pub linescore: LineScore,
    pub boxscore: TeamBatterBoxscore,
    pub current_at_bat: AtBatIndex,
    pub at_bats: IndexMap<AtBatIndex, AtBatV2>,
    pub win_probability: WinProbability,
    pub players: HashMap<u64, Player>,
}

#[derive(Default, Debug)]
pub struct PlayerStats {
    pub summary: Option<String>,
    pub note: Option<String>,
    pub pitches_thrown: Option<u8>,
    pub strikes: Option<u8>,
    #[allow(dead_code)]
    pub balls: Option<u8>,
}

impl From<&ApiPlayer> for PlayerStats {
    fn from(person: &ApiPlayer) -> Self {
        let is_pitcher = person.position.position_type == "Pitcher";
        if is_pitcher {
            Self {
                summary: person.stats.pitching.summary.clone(),
                note: person.stats.pitching.note.clone(),
                pitches_thrown: person.stats.pitching.pitches_thrown.map(|p| p as u8),
                strikes: person.stats.pitching.strikes.map(|p| p as u8),
                balls: person.stats.pitching.balls.map(|p| p as u8),
            }
        } else {
            Self {
                summary: person.stats.batting.summary.clone(),
                note: person.stats.batting.note.clone(),
                pitches_thrown: None,
                strikes: None,
                balls: None,
            }
        }
    }
}

impl GameStateV2 {
    /// Update with latest data from the API.
    pub fn update(&mut self, live_data: &LiveResponse, win_probability: &WinProbabilityResponse) {
        if self.game_id != live_data.game_pk {
            self.reset();
        }
        self.game_id = live_data.game_pk;
        self.players = Self::create_players(live_data);
        self.generate_summary(live_data);
        self.current_at_bat = Self::get_current_play_ab_index(live_data);
        self.boxscore = TeamBatterBoxscore::from_live_data(live_data, &self.players);
        self.linescore = LineScore::from_live_data(live_data);
        if let Some(plays) = &live_data.live_data.plays.all_plays {
            plays.iter().for_each(|p| Self::update_single_play(self, p));
        }
        self.win_probability = WinProbability::from(win_probability);
    }

    fn generate_summary(&mut self, live_data: &LiveResponse) {
        // self.summary = Summary::from(live_data);
        self.home_team = crate::components::constants::TEAM_IDS
            .get(live_data.game_data.teams.home.name.as_str())
            .cloned()
            .unwrap_or_default();
        self.away_team = crate::components::constants::TEAM_IDS
            .get(live_data.game_data.teams.away.name.as_str())
            .cloned()
            .unwrap_or_default();
    }

    fn create_players(live_data: &LiveResponse) -> HashMap<u64, Player> {
        let mut map = HashMap::new();
        for player in live_data.game_data.players.values() {
            map.insert(player.id, Player::from(player));
        }

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
        let at_bat = AtBatV2::from(play);
        self.at_bats.insert(at_bat.index, at_bat);
    }

    pub fn reset(&mut self) {
        *self = Self::default()
    }
}
