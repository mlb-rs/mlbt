use crate::util::convert_color;
use mlb_api::live::{LiveResponse, PlayEvent};

use crate::heatmap::{DEFAULT_SZ_BOT, DEFAULT_SZ_TOP};
use tui::style::Color;

#[derive(Debug)]
pub struct Pitch {
    pub strike: bool,
    pub color: Color,
    pub description: String, // called strike, hit, strike out, ect.
    pub location: (f64, f64),
    pub index: u8,
    pub pitch_type: String, // fastball, slider, ect.
    pub speed: f64,
    pub strike_zone_bot: f64,
    pub strike_zone_top: f64,
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
        }
    }
}

#[derive(Debug)]
pub struct Pitches {
    pub pitches: Vec<Pitch>,
}

impl Default for Pitches {
    fn default() -> Self {
        Pitches { pitches: vec![] }
    }
}

impl Pitches {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_live_data(live_game: &LiveResponse) -> Pitches {
        let pitch_data = match live_game.live_data.plays.current_play.as_ref() {
            Some(c) => &c.play_events,
            None => return Pitches::new(),
        };
        Pitches {
            pitches: Pitches::transform_pitches(pitch_data),
        }
    }

    fn transform_pitches(plays: &[PlayEvent]) -> Vec<Pitch> {
        plays
            .iter()
            .filter(|play| play.is_pitch)
            .map(|play| Pitches::transform_pitch_data(&play))
            .rev()
            .collect()
    }

    fn transform_pitch_data(play: &PlayEvent) -> Pitch {
        let pitch_data = play.pitch_data.as_ref().unwrap(); // TODO
        let pitch_coords = &pitch_data.coordinates;
        let pitch_details = &play.details;

        // x coordinate is left/right
        // z coordinate is up/down
        // y coordinate is catcher looking towards pitcher
        let x_coord = pitch_coords.get("pX").unwrap_or(&0.0);
        let z_coord = pitch_coords.get("pZ").unwrap_or(&2.0);

        Pitch {
            strike: pitch_details.is_strike.unwrap(),
            speed: pitch_data.start_speed.unwrap_or(0.0),
            color: convert_color(pitch_details.ball_color.clone().unwrap_or_default()),
            description: pitch_details.description.clone().unwrap_or_default(),
            pitch_type: pitch_details
                .pitch_type
                .clone()
                .unwrap_or_default()
                .description,
            location: (*x_coord, *z_coord),
            index: play.pitch_number.unwrap_or_default(),
            strike_zone_bot: pitch_data.strike_zone_top.unwrap_or(DEFAULT_SZ_BOT),
            strike_zone_top: pitch_data.strike_zone_top.unwrap_or(DEFAULT_SZ_TOP),
        }
    }
}

#[test]
fn test_pitches_with_defaults() {
    // Testing what happens if there is no pitch data
    let play_event = vec![PlayEvent::default()];
    let pitches = Pitches::transform_pitches(&play_event);
    assert_eq!(pitches.len(), 1);
    assert_eq!(pitches[0].description, "-".to_string());
}
