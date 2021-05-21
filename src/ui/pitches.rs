use super::super::pitches::Pitches;
use crate::pitches::Pitch;
use tui::{
    backend::Backend,
    layout::{Constraint, Corner, Direction, Layout, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::canvas::{Canvas, Rectangle},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

// TODO figure out better way to do this? used for the pitch label
static TEST: &[&str] = &[
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15",
];

impl Pitch {
    /// Convert a pitch into a TUI Rectangle so it can be displayed in a Canvas.
    pub fn as_rectangle(&self, scale: f64, ball_scale: f64) -> Rectangle {
        Rectangle {
            color: self.color,
            height: scale / ball_scale,
            width: scale / ball_scale,
            x: self.location.0 * scale,
            y: self.location.1 * scale,
        }
    }

    /// Convert a pitch into a TUI List item, displaying the pitch index, result
    /// (ball, strike, ect.), and pitch type (cutter, changeup, ect.)
    /// For example: "1  Foul | Four-Seam Fastball"
    pub fn as_list_item(&self) -> ListItem {
        ListItem::new(vec![Spans::from(vec![
            Span::styled(
                format!(" {} ", TEST[self.index as usize]),
                Style::default().fg(self.color),
            ),
            Span::raw(format!(" {} | {} ", self.description, self.pitch_type)),
        ])])
    }
}

impl Pitches {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        // TODO redo layout generation
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

        // TODO scale this correctly so it overlays the heatmap
        let scale = 13f64;
        let ball_scale = 3.0;
        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::NONE))
            .paint(|ctx| {
                for pitch in &self.pitches {
                    let ball = pitch.as_rectangle(scale, ball_scale);
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
            .x_bounds([-50.0, 100.0])
            .y_bounds([-50.0, 100.0]);

        f.render_widget(canvas, chunks[0]);

        // display the pitch information
        let pitches: Vec<ListItem> = self
            .pitches
            .iter()
            .map(|pitch| pitch.as_list_item())
            .collect();

        let events_list = List::new(pitches)
            .block(Block::default().borders(Borders::LEFT | Borders::RIGHT))
            .start_corner(Corner::TopLeft);
        f.render_widget(events_list, chunks[1]);
    }
}
