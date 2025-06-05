use crate::components::pitch_event::PitchEvent;
use crate::components::strikezone::{DEFAULT_SZ_BOT, DEFAULT_SZ_TOP};
use crate::components::util::convert_color;
use mlb_api::live::LiveResponse;
use mlb_api::plays::{Play, PlayEvent};
use tui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::canvas::Rectangle,
};

#[derive(Debug, Default)]
pub struct Pitches {
    pub pitch_events: Vec<PitchEvent>,
}

#[derive(Debug)]
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
        match live_game.live_data.plays.current_play.as_ref() {
            Some(p) => Pitches::from_play(p),
            None => Pitches::default(),
        }
    }

    pub fn from_play(play: &Play) -> Self {
        Pitches {
            pitch_events: Pitches::transform_pitch_events(&play.play_events),
        }
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
