use mlb_api::live::{LiveResponse, PlayEvent};

use tui::style::Color;

pub struct Pitch {
    pub strike: bool,
    pub color: Color,
    pub description: String, // fastball, slider, ect.
    pub location: (f64, f64),
}

#[derive(Default)]
pub struct Pitches {
    pub pitches: Vec<Pitch>,
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

                let info = &pitch_data.coordinates;
                let x_coord = info.get("pX").unwrap();
                let z_coord = info.get("pZ").unwrap();
                // x coordinate is left/right
                // z coordinate is up/down
                // y coordinate is catcher looking towards pitcher
                let pitch = Pitch {
                    strike: play.details.is_strike.unwrap(),
                    color: Pitches::convert_color(
                        play.details.ball_color.clone().unwrap_or_default(),
                    ),
                    description: play.details.description.to_string(),
                    location: (*x_coord, *z_coord),
                };
                self.pitches.push(pitch);
                // println!("{:?}", pitch_data)
            }
        }
    }

    /// Convert a string from the API to a Color::Rgb. The string starts out as:
    /// "rgba(255, 255, 255, 0.55)".
    fn convert_color(s: String) -> Color {
        if let Some(s) = s.strip_prefix("rgba(") {
            let c: Vec<&str> = s.split(", ").collect();
            Color::Rgb(
                c[0].parse().unwrap_or(0),
                c[1].parse().unwrap_or(0),
                c[2].parse().unwrap_or(0),
            )
        } else {
            eprintln!("color doesn't start with 'rgba(' {:?}", s);
            Color::Rgb(0, 0, 0)
        }
    }
}
