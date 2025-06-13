use mlb_api::plays::{Count, Play};

#[derive(Default)]
#[allow(dead_code)]
pub struct PlayEvent {
    pub code: String,
    // TODO
}

#[derive(Default)]
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

impl From<&Play> for PlayResult {
    fn from(play: &Play) -> Self {
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
