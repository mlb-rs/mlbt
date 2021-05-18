use mlb_api::live::{LiveResponse, StatElement};

use tui::style::Color;

pub struct Heatmap {
    pub cells: Vec<Color>,
    pub colors: Vec<Color>,
}

impl Default for Heatmap {
    fn default() -> Self {
        Heatmap {
            cells: Heatmap::all_black(),
            colors: Heatmap::all_black(),
        }
    }
}

impl Heatmap {
    pub fn new() -> Self {
        Heatmap {
            cells: vec![],
            colors: vec![],
        }
    }

    /// Generate a heatmap from live game data. If there is no heatmap data the
    /// heatmap will be all black.
    ///
    /// To get to the heat map zones, the API response is traversed like so:
    /// liveData > plays > currentPlay > matchup > batterHotColdZoneStats
    /// > stats > splits > stat > zones
    ///
    /// It is super nested, and unclear how robust this will be.
    pub fn from_live_data(live_game: &LiveResponse) -> Heatmap {
        let zones = match live_game.live_data.plays.current_play.as_ref() {
            Some(c) => match &c.matchup.batter_hot_cold_zone_stats.as_ref() {
                Some(z) => &z.stats,
                None => return Heatmap::default(),
            },
            None => return Heatmap::default(),
        };
        let mut heatmap = Heatmap::new();
        heatmap.transform_zones(zones);
        heatmap
    }

    /// Go through the zones and pull out the batting average colors. There are
    /// usually 13 zones that are supplied, although I'm unsure why there are
    /// that many. I am only grabbing the first 9 to create a 3x3 heatmap. My
    /// theory is that the last 4 are used for coloring the edges of the real
    /// heatmap shown on MLB GameDay?
    fn transform_zones(&mut self, zones: &[StatElement]) {
        for z in zones {
            // splits has 3 elements:
            // 0 - exit velocity
            // 1 - batting average
            // 2 - on base plus slugging
            // it's unclear if these are always ordered this way
            for split in &z.splits {
                if split.stat.name == "battingAverage" {
                    for zone in &split.stat.zones {
                        let c = Heatmap::convert_color(zone.color.clone());
                        self.cells.push(c);
                        self.colors.push(c);
                        // print!("{:?} ", c);
                    }
                    // println!();
                }
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

    fn all_black() -> Vec<Color> {
        (0..9).map(|_| Color::Rgb(0, 0, 0)).collect()
    }
}

#[test]
fn test_color_conversion() {
    let tests = vec![
        ("rgba(0, 0, 0, .55)", Color::Rgb(0, 0, 0)),
        ("rgba(6, 90, 238, .55)", Color::Rgb(6, 90, 238)),
        ("rgba(150, 188, 255, .55)", Color::Rgb(150, 188, 255)),
        ("rgba(214, 41, 52, .55)", Color::Rgb(214, 41, 52)),
        ("rgba(255, 255, 255, 0.55)", Color::Rgb(255, 255, 255)),
    ];
    for t in tests {
        assert_eq!(Heatmap::convert_color(t.0.to_string()), t.1);
    }

    let bad = ("rgba(55, 255, 255, 0.55)", Color::Rgb(255, 255, 255));
    assert_ne!(Heatmap::convert_color(bad.0.to_string()), bad.1);

    let nonsense = ("rgba(-5, 255, 255, 0.55)", Color::Rgb(0, 255, 255));
    assert_eq!(Heatmap::convert_color(nonsense.0.to_string()), nonsense.1);
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
    assert_eq!(hm.cells, good);
}

#[test]
fn test_new() {
    let hm = Heatmap::new();
    let good = vec![];
    assert_eq!(hm.cells, good);
}
