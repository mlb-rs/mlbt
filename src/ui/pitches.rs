use super::super::pitches::Pitches;
use super::utils::centered_rect;
use std::borrow::Borrow;
use tui::style::Color;
use tui::widgets::canvas::{Canvas, Points};
use tui::{
    backend::Backend,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders},
    Frame,
};

impl Pitches {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::ALL))
            .paint(|ctx| {
                for pitch in &self.pitches {
                    let ball = Points {
                        coords: &[pitch.location],
                        color: pitch.color,
                    };
                    ctx.draw(&ball);
                }
            })
            .x_bounds([-100.0, 100.0])
            .y_bounds([-100.0, 100.0]);

        // TODO figure out bounds, location of canvas, ect.
        f.render_widget(canvas, rect);
    }
}
