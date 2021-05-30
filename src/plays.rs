use mlb_api::live::LiveResponse;
use mlb_api::plays::{Count, Play};

#[derive(Default)]
pub struct InningPlays {
    pub inning: u8, // do i need this?
    pub play_results: Vec<PlayResult>,
}

#[derive(Debug)]
pub struct PlayResult {
    pub description: String,
    pub rbi: u8,
    pub away_score: u8,
    pub home_score: u8,
    pub count: Count,
    pub out: u8,
}

impl InningPlays {
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
            .map(|play| PlayResult {
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
                out: play.count.outs,
            })
            .collect();

        InningPlays {
            inning: current_inning,
            play_results: results,
        }
    }
}
