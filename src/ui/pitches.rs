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

// Used to display the pitch number next to the pitch type. Hopefully no one has at bat longer than
// 20 pitches.
const PITCH_IDX: &[&str] = &[
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
    "17", "18", "19", "20",
];
const DEFAULT_IDX: &str = "-";

impl Pitch {
    /// Convert a pitch into a TUI Rectangle so it can be displayed in a Canvas.
    pub fn as_rectangle(&self) -> Rectangle {
        let scale = 12f64; // feet to inches
        let ball_scale = 1.0;
        Rectangle {
            color: self.color,
            height: ball_scale,
            width: ball_scale,
            x: self.location.0 * scale,
            y: self.location.1 * scale,
        }
    }

    /// Convert a pitch into a TUI List item, displaying the pitch index, result
    /// (ball, strike, ect.), and pitch type (cutter, changeup, ect.)
    /// For example: "1  Foul | Four-Seam Fastball"
    pub fn as_list_item(&self, debug: bool) -> ListItem {
        ListItem::new(vec![Spans::from(vec![
            Span::styled(format!(" {} ", self.index), Style::default().fg(self.color)),
            Span::raw(self.format(debug)),
        ])])
    }

    fn format(&self, debug: bool) -> String {
        let s = format!(
            " {:<20}| {:^5.1}| {}",
            self.description, self.speed, self.pitch_type
        );
        if debug {
            return format!(" {} | {:?}", s, self.location);
        }
        s
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
            .margin(2)
            .constraints(
                [
                    Constraint::Percentage(50), // heatmap/pitches
                    Constraint::Percentage(50), // pitch info
                ]
                .as_ref(),
            )
            .split(rect);

        let total_width = 4.0 * 12.0; // 4 feet (arbitrary)
        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::NONE))
            .paint(|ctx| {
                self.pitches
                    .iter()
                    .map(|pitch| {
                        let ball = pitch.as_rectangle();
                        ctx.draw(&ball);
                        ctx.print(
                            ball.x,
                            ball.y,
                            PITCH_IDX.get(pitch.index as usize).unwrap_or(&DEFAULT_IDX),
                            pitch.color,
                        )
                    })
                    .collect()
            })
            .x_bounds([-0.5 * total_width, 0.5 * total_width])
            .y_bounds([0.0, 60.0]);

        f.render_widget(canvas, chunks[0]);

        // display the pitch information
        let pitches: Vec<ListItem> = self
            .pitches
            .iter()
            .map(|pitch| pitch.as_list_item(false))
            .collect();

        let events_list = List::new(pitches)
            .block(Block::default().borders(Borders::NONE))
            .start_corner(Corner::TopLeft);
        f.render_widget(events_list, chunks[1]);
    }
}
