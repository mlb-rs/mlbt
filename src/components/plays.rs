use crate::components::live_game::GameStateV2;
use mlb_api::live::LiveResponse;
use mlb_api::plays::{Count, Play};

#[derive(Default)]
pub struct InningPlays {
    #[allow(dead_code)]
    pub inning: u8, // do i need this?
    pub play_results: Vec<PlayResult>,
}

#[derive(Clone, Default)] // TODO remove clone
pub struct PlayEvent {
    pub code: String,
    // TODO
}

#[derive(Clone, Default)] // TODO remove clone
pub struct PlayResult {
    pub at_bat_index: u8,
    pub description: String,
    pub rbi: u8,
    pub away_score: u8,
    pub home_score: u8,
    pub count: Count,
    pub is_out: bool,
    pub is_scoring_play: bool,
    pub events: Vec<PlayEvent>,
}

impl InningPlays {
    pub fn from_gameday_v2(data: &GameStateV2) -> Self {
        let current_play = data.get_latest_at_bat();
        let inning = current_play.inning;
        let is_top_inning = current_play.is_top_inning;

        if inning == 0 {
            return InningPlays::default();
        }

        let mut play_results: Vec<PlayResult> = data
            .at_bats
            .values()
            .filter_map(|play| {
                if play.inning == inning && play.is_top_inning == is_top_inning {
                    Some(play.play_result.clone())
                } else {
                    None
                }
            })
            .collect();

        // ensure the events are sorted by index, smallest first since it gets reversed later
        play_results.sort_by_key(|p| p.at_bat_index);

        Self {
            inning,
            play_results,
        }
    }

    pub fn from_live_data(live_game: &LiveResponse) -> InningPlays {
        // get current inning, including top/bot
        let current_inning = live_game.live_data.linescore.current_inning.unwrap_or(0);
        let is_top = live_game.live_data.linescore.is_top_inning.unwrap_or(true);
        if current_inning == 0 {
            return InningPlays::default();
        }
        // get plays indices for inning
        let plays_per_inning = match live_game.live_data.plays.plays_by_inning.as_ref() {
            Some(plays) => plays,
            None => return InningPlays::default(),
        };
        let inning_info = &plays_per_inning[current_inning as usize - 1];
        let play_indices = match is_top {
            true => &inning_info.top,
            false => &inning_info.bottom,
        };
        // use indices to slice all plays
        let plays: Vec<&Play> = match live_game.live_data.plays.all_plays.as_ref() {
            Some(plays) => play_indices
                .iter()
                .filter_map(|idx| plays.get(*idx as usize))
                .collect(),
            None => return InningPlays::default(),
        };
        // construct play results from inning plays
        let results = plays
            .iter()
            .map(|play| PlayResult::from_play(play))
            .collect();

        InningPlays {
            inning: current_inning,
            play_results: results,
        }
    }
}
impl PlayResult {
    pub fn from_play(play: &Play) -> Self {
        Self {
            at_bat_index: play.about.at_bat_index,
            description: play
                .result
                .description
                .as_ref()
                .unwrap_or(&"".to_string())
                .to_string(),
            rbi: play.result.rbi.unwrap_or(0),
            away_score: play.result.away_score.unwrap_or(0),
            home_score: play.result.home_score.unwrap_or(0),
            count: play.count.clone(),
            is_out: play.result.is_out.unwrap_or(false),
            is_scoring_play: play.about.is_scoring_play.unwrap_or(false),
            events: vec![],
        }
    }
}
