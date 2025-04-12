use crate::components::strikezone::{DEFAULT_SZ_BOT, DEFAULT_SZ_TOP};
use crate::components::util::convert_color;
use crate::ui::plays::{BLUE, SCORING_SYMBOL};

use mlb_api::live::LiveResponse;
use mlb_api::plays::PlayEvent;
use tui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::canvas::Rectangle,
};

#[derive(Debug, Default)]
pub struct Pitches {
    pub pitch_events: Vec<PitchEvent>,
}

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

#[derive(Clone, Debug)]
pub struct Pitch {
    #[allow(dead_code)]
    pub strike: bool,
    pub color: Color,
    pub description: String, // called strike, hit, strike out, ect.
    pub location: (f64, f64),
    pub index: u8,
    pub pitch_type: String, // fastball, slider, ect.
    pub speed: f64,
    pub strike_zone_bot: f64,
    pub strike_zone_top: f64,
    pub count: Count,
}

impl Default for Pitch {
    fn default() -> Self {
        Pitch {
            strike: false,
            color: Color::Black,
            description: "-".to_string(),
            location: (0.0, 0.0),
            index: 0,
            pitch_type: "-".to_string(),
            speed: 0.0,
            strike_zone_bot: DEFAULT_SZ_BOT,
            strike_zone_top: DEFAULT_SZ_TOP,
            count: Count::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Count {
    pub balls: u8,
    pub strikes: u8,
}

impl From<mlb_api::plays::Count> for Count {
    fn from(value: mlb_api::plays::Count) -> Self {
        Self {
            balls: value.balls,
            strikes: value.strikes,
        }
    }
}

impl From<&PlayEvent> for PitchEvent {
    fn from(play: &PlayEvent) -> Self {
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
        if self.event_type == PitchEventType::Pitch && self.pitch.is_some() {
            Some(self.pitch.as_ref().unwrap().as_lines(debug))
        } else if self.event_type != PitchEventType::Pitch {
            let mut spans = Vec::new();
            if self.is_scoring.unwrap_or(false) {
                spans.push(Span::styled(
                    format!(" {SCORING_SYMBOL}"),
                    Style::default().fg(BLUE),
                ));
                if let (Some(away), Some(home)) = (self.away_score, self.home_score) {
                    spans.push(Span::raw(format!(" {away}-{home}")));
                }
            }
            spans.push(Span::raw(format!(" {}", self.description)));
            spans.push(Span::raw(" "));
            Some(vec![Line::from(spans)])
        } else {
            None
        }
    }
}

impl From<&PlayEvent> for Pitch {
    fn from(play: &PlayEvent) -> Self {
        let pitch_data = match play.pitch_data.as_ref() {
            Some(p) => p,
            None => return Pitch::default(),
        };
        let pitch_coords = &pitch_data.coordinates;
        let pitch_details = &play.details;

        // x coordinate is left/right
        // y coordinate is catcher looking towards pitcher
        // z coordinate is up/down
        let x_coord = pitch_coords.get("pX").unwrap_or(&0.0);
        let z_coord = pitch_coords.get("pZ").unwrap_or(&2.0);

        Pitch {
            strike: pitch_details.is_strike.unwrap_or(false),
            speed: pitch_data.start_speed.unwrap_or(0.0),
            color: convert_color(
                pitch_details
                    .ball_color
                    .clone()
                    .unwrap_or_else(|| String::from("rgba(255, 255, 255, 0)")),
            ),
            description: pitch_details.description.clone().unwrap_or_default(),
            pitch_type: pitch_details
                .pitch_type
                .clone()
                .unwrap_or_default()
                .description,
            location: (*x_coord, *z_coord),
            index: play.pitch_number.unwrap_or_default(),
            strike_zone_bot: pitch_data.strike_zone_bottom.unwrap_or(DEFAULT_SZ_BOT),
            strike_zone_top: pitch_data.strike_zone_top.unwrap_or(DEFAULT_SZ_TOP),
            count: play.count.clone().into(),
        }
    }
}

impl Pitch {
    /// Convert a pitch into a TUI Rectangle so it can be displayed in a Canvas.
    pub fn as_rectangle(&self) -> Rectangle {
        const SCALE: f64 = 12.0; // feet to inches
        const BALL_SCALE: f64 = 1.0;
        Rectangle {
            color: self.color,
            height: BALL_SCALE,
            width: BALL_SCALE,
            x: self.location.0 * SCALE,
            y: self.location.1 * SCALE,
        }
    }

    /// Convert a pitch into a TUI Line item, displaying the pitch index, result (ball, strike, ect)
    /// and pitch type (cutter, changeup, ect). For example: "1  Foul | Four-Seam Fastball"
    pub fn as_lines(&self, debug: bool) -> Vec<Line> {
        vec![Line::from(vec![
            Span::styled(format!(" {} ", self.index), Style::default().fg(self.color)),
            Span::raw(self.format(debug)),
        ])]
    }

    fn format(&self, debug: bool) -> String {
        let s = format!(
            " {:<20}| {}-{} | {:^5.1}| {}",
            self.description, self.count.balls, self.count.strikes, self.speed, self.pitch_type
        );
        if debug {
            return format!(" {} | {:?}", s, self.location);
        }
        s
    }
}

impl Pitches {
    pub fn from_live_data(live_game: &LiveResponse) -> Self {
        let pitch_events = match live_game.live_data.plays.current_play.as_ref() {
            Some(c) => Pitches::transform_pitch_events(&c.play_events),
            None => return Pitches::default(),
        };
        Pitches { pitch_events }
    }

    fn transform_pitch_events(plays: &[PlayEvent]) -> Vec<PitchEvent> {
        plays.iter().map(PitchEvent::from).rev().collect()
    }
}

#[test]
fn test_pitches_with_defaults() {
    // Testing what happens if there is no pitch data
    let play_event = vec![PlayEvent::default()];
    let pitches = Pitches::transform_pitch_events(&play_event);
    assert_eq!(pitches.len(), 1);
}
