use crate::state::date_input::DateInput;
use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

pub struct DateSelectorWidget {}

impl StatefulWidget for DateSelectorWidget {
    type State = DateInput;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let clear = Clear;
        clear.render(area, buf);

        let [_, instruct, inp] = Layout::vertical([
            Constraint::Length(1), // top border
            Constraint::Length(1), // instructions
            Constraint::Length(1), // input
        ])
        .areas(area);

        let instructions = Paragraph::new(" Press Enter to submit or Esc to cancel");
        instructions.render(instruct, buf);

        let input = Paragraph::new(format!(" {}", state.text));
        input.render(inp, buf);

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
