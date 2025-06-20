use crate::components::game::live_game::GameStateV2;
use crate::components::game::strikezone::{
    DEFAULT_SZ_BOT, DEFAULT_SZ_TOP, HOME_PLATE_WIDTH, StrikeZone,
};
use tui::prelude::*;
use tui::widgets::canvas::{Canvas, Rectangle};
use tui::widgets::{Block, Borders, Paragraph, Wrap};

pub struct AtBatWidget<'a> {
    pub game: &'a GameStateV2,
    pub selected_at_bat: Option<u8>,
}

impl Widget for AtBatWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (at_bat, _is_current) = self
            .game
            .get_at_bat_by_index_or_current(self.selected_at_bat);
        let pitches = &at_bat.pitches;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(2)
            .vertical_margin(1)
            .constraints(
                [
                    Constraint::Length(1),      // on deck
                    Constraint::Percentage(65), // heatmap/pitches
                    Constraint::Length(1),      // hit stats
                    Constraint::Percentage(35), // pitch info
                ]
                .as_ref(),
            )
            .split(area);

        // grab the strike zone from the first pitch since it doesn't change during the at bat.
        let mut strike_zone_bot = DEFAULT_SZ_BOT * 12.0;
        let mut strike_zone_top = DEFAULT_SZ_TOP * 12.0;
        for pe in &pitches.pitches.pitch_events {
            if let Some(pitch) = pe.pitch.as_ref() {
                strike_zone_bot = pitch.strike_zone_bot * 12.0;
                strike_zone_top = pitch.strike_zone_top * 12.0;
                break;
            }
        }
        let height = strike_zone_top - strike_zone_bot;
        let coords = StrikeZone::build_coords(strike_zone_bot, strike_zone_top);
        let strike_zone_area = generate_strike_zone_area(chunks[1]);

        // strike zone and pitch display
        Canvas::default()
            .block(Block::default().borders(Borders::NONE))
            .paint(|ctx| {
                for (i, coord) in coords.iter().enumerate() {
                    let r = Rectangle {
                        x: coord.0,
                        y: coord.1,
                        width: (HOME_PLATE_WIDTH / 3.0),
                        height: (height / 3.0),
                        color: pitches.strike_zone.colors[i],
                    };
                    ctx.draw(&r);
                }
                for pe in &pitches.pitches.pitch_events {
                    if let Some(pitch) = &pe.pitch {
                        let ball = pitch.as_rectangle();
                        let pitch_count = pitch.index.to_string();
                        ctx.draw(&ball);
                        ctx.print(
                            ball.x,
                            ball.y,
                            Span::styled(pitch_count, Style::default().fg(pitch.color)),
                        );
                    }
                }
            })
            .x_bounds([-25.0, 25.0])
            .y_bounds([0.0, 55.0])
            .render(strike_zone_area, buf);

        // display the on deck information if available

        // display the event information
        let events: Vec<Line> = pitches
            .pitches
            .pitch_events
            .iter()
            .rev() // reverse so that the last event is at the top
            .filter_map(|event| event.as_lines(false))
            .flatten()
            .collect();

        let text = Text::from(events);
        let paragraph = Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::TOP));
        Widget::render(paragraph, chunks[3], buf);

        // display the hit information if available
        if let Some(data) = pitches.pitches.pitch_events.last() {
            if let Some(text) = data.format_hit_data() {
                let text = Text::from(text);
                let paragraph = Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false });
                Widget::render(paragraph.clone(), chunks[2], buf);
            }
        };
    }
}

fn generate_strike_zone_area(area: Rect) -> Rect {
    // kind of arbitrary values to make the zone look good
    const STRIKE_ZONE_WIDTH: u16 = 35;
    const STRIKE_ZONE_HEIGHT: u16 = 19;
    let desired_aspect_ratio = STRIKE_ZONE_WIDTH as f64 / STRIKE_ZONE_HEIGHT as f64;

    let available_width = area.width;
    let available_height = area.height;

    // calculate what dimensions would fit while maintaining aspect ratio
    let width_constrained_height = (available_width as f64 / desired_aspect_ratio) as u16;
    let height_constrained_width = (available_height as f64 * desired_aspect_ratio) as u16;

    let (final_width, final_height) = if width_constrained_height <= available_height {
        // width is the limiting factor
        (available_width, width_constrained_height)
    } else {
        // height is the limiting factor
        (height_constrained_width, available_height)
    };

    // center the calculated dimensions
    let x_offset = (available_width - final_width) / 2;
    let y_offset = (available_height - final_height) / 2;

    Rect::new(
        area.x + x_offset,
        area.y + y_offset,
        final_width,
        final_height,
    )
}
