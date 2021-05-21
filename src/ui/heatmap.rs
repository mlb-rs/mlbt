use super::super::heatmap::Heatmap;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::canvas::{Canvas, Rectangle},
    widgets::{Block, Borders},
    Frame,
};

#[derive(Debug, PartialEq)]
struct Coordinate(f64, f64);

impl Heatmap {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(50), // heatmap/pitches
                    Constraint::Percentage(50), // pitch info
                ]
                .as_ref(),
            )
            .split(rect);

        // these should be determined by the terminal size
        let width = 30; // x
        let height = 45; // y

        let coords = build_coords(width, height);

        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::LEFT | Borders::RIGHT))
            .paint(|ctx| {
                for (i, coord) in coords.iter().enumerate() {
                    let r = Rectangle {
                        x: coord.0,
                        y: coord.1,
                        width: (width / 3) as f64,
                        height: (height / 3) as f64,
                        color: self.colors[i],
                    };
                    ctx.draw(&r);
                }
            })
            // TODO figure out bounds
            .x_bounds([-20.0, 50.0])
            .y_bounds([-20.0, 50.0]);

        f.render_widget(canvas, chunks[0]);
    }
}

/// Builds the coordinates for the 3x3 heatmap. Each coordinate represents the upper left corner of
/// a heatmap zone. A tui-rs rectangle is then built from a coordinate; its positive X axis going
/// right, and positive Y axis going down, from the coordinate.
fn build_coords(width: u16, height: u16) -> Vec<Coordinate> {
    let width_chunk = width / 3;
    let height_chunk = height / 3;

    let x_coords: Vec<_> = (0..width).step_by(width_chunk as usize).collect();
    let y_coords: Vec<_> = (0..height).step_by(height_chunk as usize).collect();

    let mut coords = Vec::new();
    for x in &x_coords {
        for y in &y_coords {
            coords.push(Coordinate(*x as f64, *y as f64));
        }
    }
    coords
}

#[test]
fn test_coords() {
    let width = 6;
    let height = 3;
    let coords = build_coords(width, height);
    println!("{:?}", coords);
    let w = vec![
        Coordinate(0.0, 0.0),
        Coordinate(0.0, 1.0),
        Coordinate(0.0, 2.0),
        Coordinate(2.0, 0.0),
        Coordinate(2.0, 1.0),
        Coordinate(2.0, 2.0),
        Coordinate(4.0, 0.0),
        Coordinate(4.0, 1.0),
        Coordinate(4.0, 2.0),
    ];
    assert_eq!(w, coords);
}
