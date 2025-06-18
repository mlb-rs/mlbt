use crate::components::game::live_game::GameStateV2;
use crate::components::game::strikezone::{
    StrikeZone, DEFAULT_SZ_BOT, DEFAULT_SZ_TOP, HOME_PLATE_WIDTH,
};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::canvas::{Canvas, Rectangle},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

pub struct AtBatWidget<'a> {
    pub game: &'a GameStateV2,
    pub selected_at_bat: Option<u8>,
}

impl Widget for AtBatWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (game, _is_current) = self
            .game
            .get_at_bat_by_index_or_current(self.selected_at_bat);
        let pitches = &game.pitches;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            // .margin(2)
            .horizontal_margin(2)
            .vertical_margin(1)
            .constraints(
                [
                    // Constraint::Length(6),  // matchup
                    Constraint::Fill(1), // heatmap/pitches
                    Constraint::Min(12), // pitch info
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
            .x_bounds([-0.5 * total_width, 0.5 * total_width])
            .y_bounds([0.0, 60.0])
            .render(strikezone[1], buf);

        // display the event information
        let events: Vec<Line> = pitches
            .pitches
            .pitch_events
            .iter()
            .filter_map(|event| event.as_lines(false))
            .flatten()
            .collect();

        let text = Text::from(events);
        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        Widget::render(paragraph, chunks[1], buf);
    }
}
