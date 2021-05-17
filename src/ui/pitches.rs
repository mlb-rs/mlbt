use super::super::pitches::Pitches;
use tui::style::Color;
use tui::widgets::canvas::{Canvas, Painter, Points, Shape};
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

/// Shape to draw a rectangle from a `Rect` with the given color
#[derive(Debug, Clone)]
pub struct Circle {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub color: Color,
}

impl Shape for Circle {
    fn draw(&self, painter: &mut Painter) {
        // TODO this doesn't look great, might want to look into drawing the circle based on the grid of the terminal
        let mut coords: Vec<(f64, f64)> = Vec::new();
        let num_points = 9;
        let chunks = 2.0 * core::f64::consts::PI / num_points as f64;
        for i in 0..num_points {
            let theta = chunks * i as f64;
            let x = (self.radius * f64::cos(theta)) + self.x;
            let y = (self.radius * f64::sin(theta)) + self.y;
            coords.push((x, y));
        }
        // TODO fill in the circle too?
        for coord in coords {
            let pt = Points {
                coords: &[coord],
                color: self.color,
            };
            pt.draw(painter)
        }
    }
}

impl Pitches {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::ALL))
            .paint(|ctx| {
                for pitch in &self.pitches {
                    let circle = Circle {
                        x: pitch.location.0,
                        y: pitch.location.1,
                        radius: 1.0,
                        color: pitch.color,
                    };
                    ctx.draw(&circle);
                    // TODO fix this
                    // let text = pitch.index.to_string().as_str();
                    // ctx.print(circle.x + 1.0, circle.y, text, Color::White)
                }
            })
            .x_bounds([-100.0, 100.0])
            .y_bounds([-100.0, 100.0]);

        // TODO figure out bounds, location of canvas, ect.
        f.render_widget(canvas, rect);
    }
}
