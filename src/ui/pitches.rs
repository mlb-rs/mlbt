use super::super::pitches::Pitches;
use tui::layout::{Constraint, Direction, Layout};
use tui::text::{Span, Spans};
use tui::widgets::{List, ListItem};
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::canvas::{Canvas, Rectangle},
    widgets::{Block, Borders},
    Frame,
};
use tui::{layout::Corner, style::Style};

// TODO figure out better way to do this? used for the pitch label
static TEST: &[&str] = &[
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15",
];

impl Pitches {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(30), // left
                    Constraint::Percentage(40), // heatmap
                    Constraint::Percentage(30), // right
                ]
                .as_ref(),
            )
            .split(rect);
        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(20),      // matchup
                    Constraint::Percentage(100), // pitch info
                ]
                .as_ref(),
            )
            .split(chunks[0]);

        // TODO scale this correctly so it overlays the heatmap
        let scale = 13f64;
        let ball_scale = 3.0;
        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
            .paint(|ctx| {
                for pitch in &self.pitches {
                    let ball = Rectangle {
                        color: pitch.color,
                        height: scale / ball_scale,
                        width: scale / ball_scale,
                        x: pitch.location.0 * scale,
                        y: pitch.location.1 * scale,
                    };
                    ctx.draw(&ball);
                    ctx.print(
                        ball.x,
                        // ball.x + (ball.height / 2.0),
                        ball.y,
                        // ball.y + (ball.width / 2.0),
                        TEST[pitch.index as usize],
                        pitch.color,
                    )
                }
            })
            .x_bounds([-100.0, 100.0])
            .y_bounds([-100.0, 100.0]);

        // TODO figure out bounds, location of canvas, ect.
        f.render_widget(canvas, chunks[1]);

        // display the pitch information
        let pitches: Vec<ListItem> = self
            .pitches
            .iter()
            .map(|pitch| {
                ListItem::new(vec![Spans::from(vec![
                    Span::styled(
                        format!("  {} ", TEST[pitch.index as usize]),
                        Style::default().fg(pitch.color), // TODO color doesn't seem to work
                    ),
                    Span::raw(format!(" {} | {} ", &pitch.description, &pitch.pitch_type)),
                ])])
            })
            .collect();

        let events_list = List::new(pitches)
            .block(Block::default().borders(Borders::LEFT))
            .start_corner(Corner::TopLeft);
        f.render_widget(events_list, left[1]);
    }
}
