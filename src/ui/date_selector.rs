use crate::state::date_input::DateInput;
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, StatefulWidget, Widget},
};

pub struct DateSelectorWidget {}

impl StatefulWidget for DateSelectorWidget {
    type State = DateInput;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let clear = Clear;
        clear.render(area, buf);

        let lines = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1), // top border
                    Constraint::Length(1), // instructions
                    Constraint::Length(1), // input
                ]
                .as_ref(),
            )
            .split(area);

        let instructions = Paragraph::new(" Press Enter to submit or Esc to cancel");
        instructions.render(lines[1], buf);

        let input = Paragraph::new(format!(" {}", state.text));
        input.render(lines[2], buf);

        let border = match state.is_valid {
            true => Style::default().fg(Color::Blue),
            false => Style::default().fg(Color::Red),
        };
        let block = Block::default()
            .title("Enter a date (YYYY-MM-DD) or use arrow keys")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border);
        block.render(area, buf);
    }
}
