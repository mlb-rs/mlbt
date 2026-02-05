use crate::components::game::pitches::Pitch;
use crate::ui::gameday::plays::{BLUE, SCORING_SYMBOL, build_scoring_span};
use tui::prelude::{Line, Span, Style};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PitchEventType {
    Pitch,
    Running,
    /// pickoff attempt, mound visit, pinch hitter, etc
    Other,
}

#[derive(Debug)]
pub struct HitData {
    pub exit_velocity: Option<f64>,
    pub launch_angle: Option<f64>,
    pub distance: Option<f64>,
    #[allow(dead_code)]
    pub hardness: Option<String>,
}

#[derive(Debug)]
pub struct PitchEvent {
    pub event_type: PitchEventType,
    pub description: String,
    pub pitch: Option<Pitch>,
    pub hit_data: Option<HitData>,
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
        let hit_data = play.hit_data.as_ref().map(|d| HitData {
            exit_velocity: d.launch_speed,
            launch_angle: d.launch_angle,
            distance: d.total_distance,
            hardness: d.hardness.clone(),
        });
        let event_type = match (play.is_pitch, play.is_base_running_play) {
            (true, _) => PitchEventType::Pitch,
            (false, Some(true)) => PitchEventType::Running,
            (false, _) => PitchEventType::Other,
        };
        Self {
            event_type,
            description: play.details.description.clone().unwrap_or_default(),
            pitch,
            hit_data,
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
    pub fn as_lines(
        &self,
        debug: bool,
        home_team_abbreviation: &'static str,
        away_team_abbreviation: &'static str,
    ) -> Option<Vec<Line<'_>>> {
        match self.event_type {
            PitchEventType::Pitch if self.pitch.is_some() => {
                self.pitch.as_ref().map(|pitch| pitch.as_lines(debug))
            }
            PitchEventType::Pitch => None,
            _ => Some(self.format_non_pitch_event(home_team_abbreviation, away_team_abbreviation)),
        }
    }

    fn format_non_pitch_event(
        &self,
        home_team_abbreviation: &'static str,
        away_team_abbreviation: &'static str,
    ) -> Vec<Line<'_>> {
        let mut spans = Vec::new();

        // Add scoring information if this is a scoring event
        let is_scoring = self.is_scoring.unwrap_or(false);
        if is_scoring {
            spans.push(Span::styled(
                format!(" {SCORING_SYMBOL}"),
                Style::default().fg(BLUE),
            ));
        }

        spans.push(Span::raw(format!(" {}", self.description)));

        // Add the score at the end of the line if available
        if is_scoring
            && let (Some(away_score), Some(home_score)) = (self.away_score, self.home_score)
        {
            spans.push(build_scoring_span(
                home_score,
                home_team_abbreviation,
                away_score,
                away_team_abbreviation,
            ));
        }

        vec![Line::from(spans)]
    }

    pub fn format_hit_data(&self) -> Option<String> {
        let mut text = String::new();
        if let Some(hit) = &self.hit_data {
            if let Some(exit_velocity) = hit.exit_velocity {
                text.push_str(&format!("exit velo: {exit_velocity}"));
            }

            if let Some(launch_angle) = hit.launch_angle {
                if !text.is_empty() {
                    text.push_str(" | ");
                }
                text.push_str(&format!("LA: {launch_angle}°"));
            }

            if let Some(distance) = hit.distance {
                if !text.is_empty() {
                    text.push_str(" | ");
                }
                text.push_str(&format!("distance: {distance}'"));
            }
            Some(text)
        } else {
            None
        }
    }
}
