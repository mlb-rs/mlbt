use crate::components::game::live_game::GameStateV2;
use mlb_api::plays::{Count, Play};

#[derive(Default)]
pub struct InningPlays {
    #[allow(dead_code)]
    pub inning: u8, // do i need this?
    pub play_results: Vec<PlayResult>,
}

#[derive(Clone, Default)] // TODO remove clone
#[allow(dead_code)]
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
    pub fn from_gameday_v2(game: &GameStateV2, selected_at_bat: Option<u8>) -> Self {
        let (at_bat, _is_current) = game.get_at_bat_by_index_or_current(selected_at_bat);
        let inning = at_bat.inning;
        let is_top_inning = at_bat.is_top_inning;

        if inning == 0 {
            return InningPlays::default();
        }

        let mut play_results: Vec<PlayResult> = game
            .at_bats
            .values()
            .filter_map(|play| {
                if play.inning == inning && play.is_top_inning == is_top_inning {
                    Some(play.play_result.clone()) // TODO remove clone
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
