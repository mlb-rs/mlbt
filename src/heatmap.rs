use crate::util::convert_color;
use mlb_api::live::LiveResponse;
use mlb_api::plays::Zone;
use tui::style::Color;

pub const DEFAULT_SZ_BOT: f64 = 1.5; // feet
pub const DEFAULT_SZ_TOP: f64 = 3.3; // feet

pub struct Heatmap {
    pub colors: Vec<Color>,
    pub strike_zone_bot: f64,
    pub strike_zone_top: f64,
}

impl Default for Heatmap {
    fn default() -> Self {
        Heatmap {
            colors: Heatmap::all_black(),
            strike_zone_bot: DEFAULT_SZ_BOT,
            strike_zone_top: DEFAULT_SZ_TOP,
        }
    }
}

impl Heatmap {
    pub fn new(colors: Vec<Color>) -> Self {
        Heatmap {
            colors,
            strike_zone_bot: DEFAULT_SZ_BOT,
            strike_zone_top: DEFAULT_SZ_TOP,
        }
    }

    /// Generate a heatmap from live game data. If there is no heatmap data the
    /// heatmap will be all black.
    ///
    /// To get to the heat map zones, the API response is traversed like so:
    /// liveData > plays > currentPlay > matchup > batterHotColdZones > zones
    pub fn from_live_data(live_game: &LiveResponse) -> Heatmap {
        let colors = match live_game.live_data.plays.current_play.as_ref() {
            Some(c) => match c.matchup.batter_hot_cold_zones.as_ref() {
                Some(z) => Heatmap::transform_zones(z),
                None => return Heatmap::default(),
            },
            None => return Heatmap::default(),
        };
        Heatmap::new(colors)
    }

    /// Go through the zones and pull out the batting average colors. There are
    /// usually 13 zones that are supplied, although I'm unsure why there are
    /// that many. I am only using the first 9 to create a 3x3 heatmap. My
    /// theory is that the last 4 are used for coloring the edges of the real
    /// heatmap shown on MLB Gameday?
    fn transform_zones(zones: &[Zone]) -> Vec<Color> {
        zones
            .iter()
            .map(|z| convert_color(z.color.clone()))
            .collect()
    }

    fn all_black() -> Vec<Color> {
        (0..9).map(|_| Color::Rgb(0, 0, 0)).collect()
    }
}

#[test]
fn test_all_black() {
    let hm = Heatmap::default();
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
    let hm = Heatmap::new(vec![]);
    let good = vec![];
    assert_eq!(hm.colors, good);
}
