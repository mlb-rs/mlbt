use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

/// A centered popup with a title, instruction line, and text input.
/// Used by both the date picker and the player search.
pub struct InputPopup<'a> {
    pub title: &'a str,
    pub instructions: &'a str,
    pub input_text: &'a str,
    pub border_color: Color,
    pub info: Option<&'a str>,
}

impl Widget for InputPopup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let [_, instruct, inp] = Layout::vertical([
            Constraint::Length(1), // top border
            Constraint::Length(1), // instructions
            Constraint::Length(1), // input
        ])
        .areas(area);

        Paragraph::new(format!(" {}", self.instructions)).render(instruct, buf);
        Paragraph::new(format!(" {}", self.input_text)).render(inp, buf);
        if let Some(info) = self.info {
            let info_width = info.len() as u16 + 2; // +2 for padding
            let input_width = self.input_text.len() as u16 + 2; // +2 for " " prefix and gap
            if inp.width > info_width + input_width {
                Paragraph::new(format!("{} ", info))
                    .alignment(Alignment::Right)
                    .style(Style::default().fg(Color::DarkGray))
                    .render(inp, buf);
            }
        }

        Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.border_color))
            .render(area, buf);
    }
}

/// Create a centered popup rect with the given height and width percentage.
pub fn create_popup(area: Rect, height: u16, percent_width: u16) -> Rect {
    let [_, popup, _] = Layout::vertical(
        [
            Constraint::Ratio(1, 2),
            Constraint::Length(height),
            Constraint::Ratio(1, 2),
        ]
        .as_ref(),
    )
    .areas(area);

    Layout::horizontal(
        [
            Constraint::Percentage((100 - percent_width) / 2),
            Constraint::Percentage(percent_width),
            Constraint::Percentage((100 - percent_width) / 2),
        ]
        .as_ref(),
    )
    .split(popup)[1]
}

/// Standard cursor position for an InputPopup.
pub fn popup_cursor_position(popup_rect: Rect, input_len: u16) -> (u16, u16) {
    (
        popup_rect.x + input_len + 1, // +1 for border
        popup_rect.y + 2,             // +2 for border and instruction line
    )
}
