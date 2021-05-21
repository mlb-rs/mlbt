use crate::util::convert_color;
use mlb_api::live::{LiveResponse, PlayEvent};

use tui::style::Color;

#[derive(Debug)]
pub struct Pitch {
    pub strike: bool,
    pub color: Color,
    pub description: String, // called strike, hit, strike out, ect.
    pub location: (f64, f64),
    pub index: u8,
    pub pitch_type: String, // fastball, slider, ect.
}

#[derive(Debug)]
pub struct Pitches {
    pub pitches: Vec<Pitch>,
}

impl Default for Pitches {
    fn default() -> Self {
        Pitches {
            pitches: vec![Pitch {
                strike: false,
                color: Color::Black,
                description: "no pitch".to_string(),
                location: (0.0, 0.0),
                index: 0,
                pitch_type: "no pitch".to_string(),
            }],
        }
    }
}

impl Pitches {
    pub fn new() -> Self {
        Pitches { pitches: vec![] }
    }

    pub fn from_live_data(live_game: &LiveResponse) -> Pitches {
        let pitch_data = match live_game.live_data.plays.current_play.as_ref() {
            Some(c) => &c.play_events,
            None => return Pitches::default(),
        };
        let mut pitches = Pitches::new();
        pitches.transform_pitches(pitch_data);
        pitches
    }

    fn transform_pitches(&mut self, plays: &[PlayEvent]) {
        for play in plays {
            if play.is_pitch {
                let pitch_data = play.pitch_data.as_ref().unwrap(); // TODO
                let pitch_coords = &pitch_data.coordinates;
                let pitch_details = &play.details;

                // x coordinate is left/right
                // z coordinate is up/down
                // y coordinate is catcher looking towards pitcher
                let x_coord = pitch_coords.get("pX").unwrap();
                let z_coord = pitch_coords.get("pZ").unwrap();

                let pitch = Pitch {
                    strike: pitch_details.is_strike.unwrap(),
                    color: convert_color(pitch_details.ball_color.clone().unwrap_or_default()),
                    description: pitch_details.description.to_string(),
                    pitch_type: pitch_details
                        .pitch_type
                        .clone()
                        .unwrap_or_default()
                        .description
                        .clone(),
                    location: (*x_coord, *z_coord),
                    index: play.pitch_number.unwrap_or_default(),
                };
                self.pitches.push(pitch);
            }
        }
    }
}

#[test]
fn test_pitches_with_defaults() {
    // Testing what happens if there is no pitch data - TODO
    let play_event = vec![PlayEvent::default()];
    let mut pitches = Pitches::new();
    pitches.transform_pitches(&play_event);
}
