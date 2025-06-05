use crate::components::pitches::Pitch;
use crate::ui::gameday::plays::{BLUE, SCORING_SYMBOL};
use tui::prelude::{Line, Span, Style};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PitchEventType {
    Pitch,
    Running,
    /// pickoff attempt, mound visit, pinch hitter, etc
    Other,
}

#[derive(Debug)]
pub struct PitchEvent {
    pub event_type: PitchEventType,
    pub description: String,
    pub pitch: Option<Pitch>,
    pub is_scoring: Option<bool>,
    pub away_score: Option<u8>,
    pub home_score: Option<u8>,
}

impl From<&mlb_api::plays::PlayEvent> for PitchEvent {
    fn from(play: &mlb_api::plays::PlayEvent) -> Self {
        let pitch = match play.is_pitch {
            true => Some(Pitch::from(play)),
            false => None,
        };
        let event_type = match (play.is_pitch, play.is_base_running_play) {
            (true, _) => PitchEventType::Pitch,
            (false, Some(true)) => PitchEventType::Running,
            (false, _) => PitchEventType::Other,
        };
        Self {
            event_type,
            description: play.details.description.clone().unwrap_or_default(),
            pitch,
            is_scoring: play.details.is_scoring_play,
            away_score: play.details.away_score,
            home_score: play.details.home_score,
        }
    }
}

impl PitchEvent {
    /// Convert a pitch event into a TUI Line item.
    /// If it's a pitch, display the pitch information.
    /// Otherwise, display the description.
    pub fn as_lines(&self, debug: bool) -> Option<Vec<Line>> {
        match self.event_type {
            PitchEventType::Pitch if self.pitch.is_some() => {
                self.pitch.as_ref().map(|pitch| pitch.as_lines(debug))
            }
            PitchEventType::Pitch => None,
            _ => Some(self.format_non_pitch_event()),
        }
    }

    fn format_non_pitch_event(&self) -> Vec<Line> {
        let mut spans = Vec::new();

        // Add scoring information if this is a scoring event
        if self.is_scoring.unwrap_or(false) {
            spans.push(Span::styled(
                format!(" {SCORING_SYMBOL}"),
                Style::default().fg(BLUE),
            ));

            // Add the score if available
            if let (Some(away), Some(home)) = (self.away_score, self.home_score) {
                spans.push(Span::raw(format!(" {away}-{home}")));
            }
        }

        spans.push(Span::raw(format!(" {}", self.description)));
        spans.push(Span::raw(" "));

        vec![Line::from(spans)]
    }
}
