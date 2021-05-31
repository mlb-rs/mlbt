use crate::strikezone::{DEFAULT_SZ_BOT, DEFAULT_SZ_TOP};
use crate::util::convert_color;

use mlb_api::live::LiveResponse;
use mlb_api::plays::PlayEvent;
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
    widgets::canvas::Rectangle,
    widgets::ListItem,
};

/// Used to display the pitch number next to the pitch type in the Canvas. Hopefully no one has at
/// bat longer than 21 pitches.
/// There have been eight at bats that were 18 pitches or more since 1988, with the longest at 21.
pub const PITCH_IDX: &[&str] = &[
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
    "17", "18", "19", "20", "21",
];
pub const DEFAULT_IDX: &str = "-";

#[derive(Debug)]
pub struct Pitches {
    pub pitches: Vec<Pitch>,
}

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

impl Pitch {
    pub fn from_play(play: &PlayEvent) -> Self {
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
            strike_zone_bot: pitch_data.strike_zone_bottom.unwrap_or(DEFAULT_SZ_BOT),
            strike_zone_top: pitch_data.strike_zone_top.unwrap_or(DEFAULT_SZ_TOP),
        }
    }

    /// Convert a pitch into a TUI Rectangle so it can be displayed in a Canvas.
    pub fn as_rectangle(&self) -> Rectangle {
        let scale = 12f64; // feet to inches
        let ball_scale = 1.0;
        Rectangle {
            color: self.color,
            height: ball_scale,
            width: ball_scale,
            x: self.location.0 * scale,
            y: self.location.1 * scale,
        }
    }

    /// Convert a pitch into a TUI List item, displaying the pitch index, result (ball, strike, ect)
    /// and pitch type (cutter, changeup, ect). For example:
    /// "1  Foul | Four-Seam Fastball"
    pub fn as_list_item(&self, debug: bool) -> ListItem {
        ListItem::new(vec![Spans::from(vec![
            Span::styled(format!(" {} ", self.index), Style::default().fg(self.color)),
            Span::raw(self.format(debug)),
        ])])
    }

    fn format(&self, debug: bool) -> String {
        let s = format!(
            " {:<20}| {:^5.1}| {}",
            self.description, self.speed, self.pitch_type
        );
        if debug {
            return format!(" {} | {:?}", s, self.location);
        }
        s
    }
}

impl Default for Pitches {
    fn default() -> Self {
        Pitches { pitches: vec![] }
    }
}

impl Pitches {
    pub fn new(pitches: Vec<Pitch>) -> Self {
        Pitches { pitches }
    }

    pub fn from_live_data(live_game: &LiveResponse) -> Self {
        let pitch_data = match live_game.live_data.plays.current_play.as_ref() {
            Some(c) => Pitches::transform_pitches(&c.play_events),
            None => return Pitches::default(),
        };
        Pitches::new(pitch_data)
    }

    fn transform_pitches(plays: &[PlayEvent]) -> Vec<Pitch> {
        plays
            .iter()
            .filter(|play| play.is_pitch)
            .map(|play| Pitch::from_play(&play))
            .rev()
            .collect()
    }
}

#[test]
fn test_pitches_with_defaults() {
    // Testing what happens if there is no pitch data
    let play_event = vec![PlayEvent::default()];
    let pitches = Pitches::transform_pitches(&play_event);
    assert_eq!(pitches.len(), 0);
}
