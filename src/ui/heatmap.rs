use super::super::heatmap::Heatmap;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::canvas::{Canvas, Rectangle},
    widgets::{Block, Borders},
    Frame,
};

const HOME_PLATE_WIDTH: f64 = 17.0; // inches
lazy_static! {
    // Create the x coordinates for the heat map zones based on the width of home plate, which is 17
    // inches. The coordinates are centered around 0 in the x, thus the first coordinate is all the
    // way to the left at -8.5. Then just add (17 / 3) for the next two coordinates, or divide by 6.
    static ref X_COORDS: Vec<f64> = vec![-8.5, 17.0 / -6.0, 17.0 / 6.0];
}

#[derive(Debug, PartialEq)]
struct Coordinate(f64, f64);

impl Heatmap {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(3)
            .constraints(
                [
                    Constraint::Percentage(50), // heatmap/pitches
                    Constraint::Percentage(50), // pitch info
                ]
                .as_ref(),
            )
            .split(rect);

        // these should be determined by the terminal size

        let total_width = 4.0 * 12.0; // 4 feet (arbitrary)

        // TODO sz top and bot are specified per player
        let strike_zone_bot = self.strike_zone_bot * 12.0; // feet
        let strike_zone_top = self.strike_zone_top * 12.0; // feet
        let height = strike_zone_top - strike_zone_bot;

        let coords = build_coords(strike_zone_bot, strike_zone_top);

        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::NONE))
            .paint(|ctx| {
                for (i, coord) in coords.iter().enumerate() {
                    let r = Rectangle {
                        x: coord.0,
                        y: coord.1,
                        width: (HOME_PLATE_WIDTH / 3.0) as f64,
                        height: (height / 3.0) as f64,
                        color: self.colors[i],
                    };
                    ctx.draw(&r);
                }
            })
            .x_bounds([-0.5 * total_width, 0.5 * total_width])
            .y_bounds([0.0, 60.0]);

        f.render_widget(canvas, chunks[0]);
    }
}

/// Builds the coordinates for the 3x3 heatmap. Each coordinate represents the upper left corner of
/// a heatmap zone. A tui-rs rectangle is then built from a coordinate; its positive X axis going
/// right, and positive Y axis going down, from the coordinate.
fn build_coords(strike_zone_bot: f64, strike_zone_top: f64) -> Vec<Coordinate> {
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

#[test]
fn test_coords() {
    let bot = 1.5 * 12.0;
    let top = 3.3 * 12.0;
    let coords = build_coords(bot, top);
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
