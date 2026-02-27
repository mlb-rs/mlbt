use crate::components::game::live_game::PlayerMap;
use crate::components::game::pitch_event::PitchEvent;
use crate::components::game::review::ReviewDetails;
use crate::components::game::strikezone::{DEFAULT_SZ_BOT, DEFAULT_SZ_TOP};
use crate::components::standings::Team;
use crate::components::util::convert_color;
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
    pub description: String, // called strike, hit, strike out, etc.
    pub location: (f64, f64),
    pub index: u8,
    pub pitch_type: String, // fastball, slider, etc.
    pub speed: f64,
    pub strike_zone_bot: f64,
    pub strike_zone_top: f64,
    pub count: Count,
    pub review_details: Option<ReviewDetails>,
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
            review_details: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
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
                .as_ref()
                .map(|pt| pt.description.clone())
                .unwrap_or_default(),
            location: (*x_coord, *z_coord),
            index: play.pitch_number.unwrap_or_default(),
            strike_zone_bot: pitch_data.strike_zone_bottom.unwrap_or(DEFAULT_SZ_BOT),
            strike_zone_top: pitch_data.strike_zone_top.unwrap_or(DEFAULT_SZ_TOP),
            count: play.count.clone().into(),
            review_details: play.review_details.as_ref().map(ReviewDetails::from),
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
    /// If the pitch is under review, a new line will displayed under the pitch with the player and
    /// team that initiated the review.
    pub fn as_lines(
        &self,
        debug: bool,
        home_team: &Team,
        away_team: &Team,
        players: &PlayerMap,
    ) -> Vec<Line<'_>> {
        let mut lines = Vec::new();

        // if the pitch is under review, display the review status above the pitch info
        if let Some(review) = &self.review_details
            && let Some(spans) = review.format_in_progress_spans(home_team, away_team, players)
        {
            let mut in_progress_spans = vec![Span::raw(" ")];
            in_progress_spans.extend(spans);
            lines.push(Line::from(in_progress_spans));
        }

        let mut pitch_line_spans = vec![Span::styled(
            format!(" {:<2}", self.index),
            Style::default().fg(self.color),
        )];
        pitch_line_spans.extend(self.format_spans(debug, players));
        lines.push(Line::from(pitch_line_spans));

        lines
    }

    fn format_spans(&self, debug: bool, players: &PlayerMap) -> Vec<Span<'_>> {
        let base = format!(
            " {:<20}| {}-{} | {:^5.1}| {}",
            self.description, self.count.balls, self.count.strikes, self.speed, self.pitch_type,
        );
        let s = if debug {
            format!(" {} | {:?}", base, self.location)
        } else {
            base
        };
        let mut spans = vec![Span::raw(s)];
        if let Some(review) = &self.review_details
            && let Some(review_spans) = review.format_status_spans(players)
        {
            spans.extend(review_spans);
        }
        spans
    }
}

impl From<&Play> for Pitches {
    fn from(play: &Play) -> Self {
        let pitch_events = play.play_events.iter().map(PitchEvent::from).collect();
        Pitches { pitch_events }
    }
}
