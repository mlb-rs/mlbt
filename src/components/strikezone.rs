use tui::style::Color;

use mlb_api::live::LiveResponse;
use mlb_api::plays::Zone;

use crate::components::util::convert_color;

/// Create the x coordinates for the heat map zones based on the width of home plate, which is 17
/// inches. The coordinates are centered around 0 in the x, thus the first coordinate is all the
/// way to the left at -8.5. Then just add (17 / 3) for the next two coordinates, or divide by 6.
pub const HOME_PLATE_WIDTH: f64 = 17.0; // inches
const X_COORDS: [f64; 3] = [-8.5, 17.0 / -6.0, 17.0 / 6.0];

/// The default strike zone bottom and top represent the horizontal bounds for the strike zone. They
/// are measured in feet from the ground. Note that the MLB considers this the z-axis (not y), with
/// the ground being z = 0.
pub const DEFAULT_SZ_BOT: f64 = 1.5; // feet
pub const DEFAULT_SZ_TOP: f64 = 3.3; // feet

#[derive(Debug, PartialEq)]
pub struct Coordinate(pub f64, pub f64);

pub struct StrikeZone {
    pub colors: Vec<Color>,
    pub strike_zone_bot: f64,
    pub strike_zone_top: f64,
}

impl Default for StrikeZone {
    fn default() -> Self {
        StrikeZone {
            colors: StrikeZone::all_black(),
            strike_zone_bot: DEFAULT_SZ_BOT,
            strike_zone_top: DEFAULT_SZ_TOP,
        }
    }
}

impl StrikeZone {
    pub fn new(colors: Vec<Color>) -> Self {
        StrikeZone {
            colors,
            strike_zone_bot: DEFAULT_SZ_BOT,
            strike_zone_top: DEFAULT_SZ_TOP,
        }
    }

    /// Generate the strike zone from the current at bat. If there is no data the strike zone will
    /// be all black.
    ///
    /// To get to the heat map zones, the API response is traversed like so:
    /// liveData > plays > currentPlay > matchup > batterHotColdZones > zones
    pub fn from_live_data(live_game: &LiveResponse) -> Self {
        let colors = match live_game.live_data.plays.current_play.as_ref() {
            Some(c) => match c.matchup.batter_hot_cold_zones.as_ref() {
                Some(z) => StrikeZone::transform_zones(z),
                None => return StrikeZone::default(),
            },
            None => return StrikeZone::default(),
        };
        // TODO set strike zone top/bottom here
        if colors.len() < 9 {
            return StrikeZone::default();
        }
        StrikeZone::new(colors)
    }

    /// Go through the zones and pull out the batting average colors. There are usually 13 zones
    /// that are supplied, although I'm unsure why there are that many. I am only using the first 9
    /// to create a 3x3 heatmap. My theory is that the last 4 are used for coloring the edges of the
    /// real heatmap shown on MLB Gameday?
    fn transform_zones(zones: &[Zone]) -> Vec<Color> {
        zones
            .iter()
            .map(|z| convert_color(z.color.clone()))
            .collect()
    }

    /// Builds the coordinates for the 3x3 heatmap. Each coordinate represents the upper left corner
    /// of a heatmap zone. A tui-rs rectangle is then built from a coordinate; its positive X axis
    /// going right, and positive Y axis going down, from the coordinate.
    pub fn build_coords(strike_zone_bot: f64, strike_zone_top: f64) -> Vec<Coordinate> {
        let y_chunk = (strike_zone_top - strike_zone_bot) / 3.0;
        let y_coords = vec![
            strike_zone_bot + (2.0 * y_chunk),
            strike_zone_bot + y_chunk,
            strike_zone_bot,
        ];

        y_coords
            .iter()
            .flat_map(|y| X_COORDS.iter().map(move |x| Coordinate(*x, *y)))
            .collect()
    }

    fn all_black() -> Vec<Color> {
        (0..9).map(|_| Color::Rgb(0, 0, 0)).collect()
    }
}

#[test]
fn test_all_black() {
    let hm = StrikeZone::default();
    let good = vec![
        Color::Rgb(0, 0, 0),
        Color::Rgb(0, 0, 0),
        Color::Rgb(0, 0, 0),
        Color::Rgb(0, 0, 0),
        Color::Rgb(0, 0, 0),
        Color::Rgb(0, 0, 0),
        Color::Rgb(0, 0, 0),
        Color::Rgb(0, 0, 0),
        Color::Rgb(0, 0, 0),
    ];
    assert_eq!(hm.colors, good);
}

#[test]
fn test_new() {
    let hm = StrikeZone::new(vec![]);
    let good = vec![];
    assert_eq!(hm.colors, good);
}

#[test]
fn test_coords() {
    let bot = 1.5 * 12.0;
    let top = 3.3 * 12.0;
    let coords = StrikeZone::build_coords(bot, top);
    let w = vec![
        Coordinate(-8.5, 32.4),
        Coordinate(17.0 / -6.0, 32.4),
        Coordinate(17.0 / 6.0, 32.4),
        Coordinate(-8.5, 25.2),
        Coordinate(17.0 / -6.0, 25.2),
        Coordinate(17.0 / 6.0, 25.2),
        Coordinate(-8.5, 18.0),
        Coordinate(17.0 / -6.0, 18.0),
        Coordinate(17.0 / 6.0, 18.0),
    ];
    assert_eq!(w, coords);
}
