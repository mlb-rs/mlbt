use crate::components::plays::{InningPlays, PlayResult};
use crate::ui::layout::LayoutAreas;

use tui::{
    buffer::Buffer,
    layout::{Corner, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, StatefulWidget, Widget},
};

// These colors match the red, green, and blue used in the pitch data from the API.
// Red is used for strikes, green for balls, and blue for contact (hit, out, run scoring).
const RED: Color = Color::Rgb(170, 21, 11);
const GREEN: Color = Color::Rgb(39, 161, 39);
const BLUE: Color = Color::Rgb(26, 86, 190);

impl InningPlays {
    pub fn as_list(&self) -> Vec<ListItem> {
        self.play_results
            .iter()
            .filter(|play| !play.description.is_empty())
            .map(|play| {
                // if runs were scored display "!!" colored blue
                let alert = match play.rbi {
                    0 => Span::raw(""),
                    _ => Span::styled("!! ", Style::default().fg(BLUE)),
                };
                ListItem::new(vec![
                    Line::from(vec![alert, Span::raw(&play.description)]),
                    Line::from(InningPlays::format_info(play)),
                ])
            })
            .rev()
            .collect()
    }

    // Depending on the outcome of the play, style the information differently. If there is a walk
    // display the 4 as green, if there is a strikeout display the 3 as red.
    fn format_info(play: &PlayResult) -> Vec<Span> {
        let runs = match play.rbi {
            0 => " ".to_string(),
            _ => format!("  runs: {}", play.rbi),
        };
        let balls = match play.count.balls {
            4 => Span::styled("4", Style::default().fg(GREEN)),
            _ => Span::from(play.count.balls.to_string()),
        };
        let strikes = match play.count.strikes {
            3 => Span::styled("3", Style::default().fg(RED)),
            _ => Span::from(play.count.strikes.to_string()),
        };
        vec![
            Span::raw(format!("{} outs: {} balls: ", runs, &play.count.outs)),
            balls,
            Span::raw(" strikes: "),
            strikes,
        ]
    }
}

pub struct InningPlaysWidget {}

impl StatefulWidget for InningPlaysWidget {
    type State = InningPlays;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunks = LayoutAreas::for_info(area);

        Widget::render(
            List::new(state.as_list())
                .block(Block::default().borders(Borders::NONE))
                .start_corner(Corner::TopLeft),
            chunks[1],
            buf,
        );
    }
}
