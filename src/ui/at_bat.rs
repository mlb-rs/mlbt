use crate::components::at_bat::AtBat;
use crate::components::pitches::{DEFAULT_IDX, PITCH_IDX};
use crate::components::strikezone::{StrikeZone, DEFAULT_SZ_BOT, DEFAULT_SZ_TOP, HOME_PLATE_WIDTH};

use tui::{
    buffer::Buffer,
    layout::{Constraint, Corner, Direction, Layout, Rect},
    style::Style,
    text::Span,
    widgets::canvas::{Canvas, Rectangle},
    widgets::{Block, Borders, List, ListItem, StatefulWidget, Widget},
};

pub struct AtBatWidget {}

impl StatefulWidget for AtBatWidget {
    type State = AtBat;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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
            .split(area);

        let total_width = 4.0 * 12.0; // 4 feet (arbitrary)

        // Constrain and center the strikezone and pitch display. Without this they get stretched
        // on wider terminals. This does, unfortunately, over compress when the terminal is small.
        // TODO when terminal width is too small, don't apply these constraints
        let strikezone = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - total_width as u16) / 2),
                    Constraint::Percentage(total_width as u16),
                    Constraint::Percentage((100 - total_width as u16) / 2),
                ]
                .as_ref(),
            )
            .split(chunks[0]);

        // grab the strike zone from the first pitch since it doesn't change during the at bat.
        let (strike_zone_bot, strike_zone_top) = match state.pitches.pitches.first() {
            Some(p) => (p.strike_zone_bot * 12.0, p.strike_zone_top * 12.0),
            None => (DEFAULT_SZ_BOT * 12.0, DEFAULT_SZ_TOP * 12.0),
        };
        let height = strike_zone_top - strike_zone_bot;
        let coords = StrikeZone::build_coords(strike_zone_bot, strike_zone_top);

        // strike zone and pitch display
        Canvas::default()
            .block(Block::default().borders(Borders::NONE))
            .paint(|ctx| {
                for pitch in &state.pitches.pitches {
                    let ball = pitch.as_rectangle();
                    let pitch_count = PITCH_IDX.get(pitch.index as usize).unwrap_or(&DEFAULT_IDX);
                    ctx.draw(&ball);
                    ctx.print(
                        ball.x,
                        ball.y,
                        Span::styled(*pitch_count, Style::default().fg(pitch.color)),
                    )
                }
                ctx.layer();
                for (i, coord) in coords.iter().enumerate() {
                    let r = Rectangle {
                        x: coord.0,
                        y: coord.1,
                        width: (HOME_PLATE_WIDTH / 3.0),
                        height: (height / 3.0),
                        color: state.strike_zone.colors[i],
                    };
                    ctx.draw(&r);
                }
            })
            .x_bounds([-0.5 * total_width, 0.5 * total_width])
            .y_bounds([0.0, 60.0])
            .render(strikezone[1], buf);

        // display the pitch information
        let pitches: Vec<ListItem> = state
            .pitches
            .pitches
            .iter()
            .map(|pitch| pitch.as_list_item(false))
            .collect();

        Widget::render(
            List::new(pitches)
                .block(Block::default().borders(Borders::NONE))
                .start_corner(Corner::TopLeft),
            chunks[1],
            buf,
        );
    }
}
