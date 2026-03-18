use crate::state::date_input::DateInput;
use crate::ui::input_popup::InputPopup;
use tui::prelude::*;
use tui::style::Color;

pub struct DateSelectorWidget {}

impl StatefulWidget for DateSelectorWidget {
    type State = DateInput;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let border_color = if state.is_valid {
            Color::Blue
        } else {
            Color::Red
        };
        InputPopup {
            title: "Enter a date (YYYY-MM-DD) or use arrow keys",
            instructions: "Press Enter to submit or Esc to cancel",
            input_text: &state.text,
            border_color,
            info: None,
        }
        .render(area, buf);
    }
}
