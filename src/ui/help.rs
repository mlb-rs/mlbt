use crate::components::banner::BANNER;
use crate::config::ConfigFile;
use tui::layout::{Alignment, Constraint, Flex, Layout};
use tui::prelude::*;
use tui::widgets::{Paragraph, Row, Table, TableState};

const HEADER: &[&str; 2] = &["Description", "Key"];
pub const DOCS: &[&[&str; 2]; 44] = &[
    &["Exit help", "Esc"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Quit", "q"],
    &["Full screen", "f"],
    &["Scoreboard", "1"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Select date", ":"],
    &["Switch boxscore team", "h/a"],
    &["Scroll boxscore down", "Shift + j/↓"],
    &["Scroll boxscore up", "Shift + k/↑"],
    &["Toggle win probability", "w"],
    &["Gameday", "2"],
    &["Toggle game info", "i"],
    &["Toggle pitches", "p"],
    &["Toggle boxscore", "b"],
    &["Switch boxscore team", "h/a"],
    &["Scroll boxscore down", "Shift + j/↓"],
    &["Scroll boxscore up", "Shift + k/↑"],
    &["Toggle win probability", "w"],
    &["Move down at bat", "j/↓"],
    &["Move up at bat", "k/↑"],
    &["Go to live at bat", "l"],
    &["Go to first at bat", "s"],
    &["Stats", "3"],
    &["Switch hitting/pitching", "h/p"],
    &["Switch team/player", "t/l"],
    &["Switch pane", "←/→/Tab"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Page down", "Shift + j/↓"],
    &["Page up", "Shift + k/↑"],
    &["Toggle stat", "Enter"],
    &["Sort by stat", "s"],
    &["Select date", ":"],
    &["Toggle stat selection", "o"],
    &["Search", "Ctrl + f"],
    &["Standings", "4"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Select date", ":"],
    &["View team info", "Enter"],
    &["Toggle division/league", "l"],
];

/// Used to keep track of whether a row should be styled like a header.
struct HelpRow {
    is_header: bool,
    text: Vec<String>,
}

pub struct HelpState {
    pub state: TableState,
}

impl Default for HelpState {
    fn default() -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        Self { state }
    }
}

impl HelpState {
    pub fn next(&mut self) {
        self.state.scroll_down_by(1);
    }

    pub fn previous(&mut self) {
        self.state.scroll_up_by(1);
    }
}

pub struct HelpWidget {}

impl StatefulWidget for HelpWidget {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create a one-column table to avoid flickering due to non-determinism when
        // resolving constraints on widths of table columns.
        let format_row = |r: &[&str; 2]| -> HelpRow {
            HelpRow {
                is_header: r[1].parse::<u8>().is_ok(),
                text: vec![format!("{:30}{:15}", r[0], r[1])],
            }
        };
        let header_style = Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
        let help_menu_style = Style::default();

        let header = Row::new(format_row(HEADER).text)
            .height(1)
            .bottom_margin(0)
            .style(header_style);

        let rows = DOCS
            .iter()
            .map(|d| format_row(d))
            .map(|item| match item.is_header {
                true => Row::new(item.text).style(header_style),
                false => Row::new(item.text).style(help_menu_style),
            });

        let [table, banner] = Layout::horizontal([Constraint::Length(50), Constraint::Length(15)])
            .flex(Flex::Legacy)
            .margin(1)
            .horizontal_margin(2)
            .areas(area);

        let selected_style = Style::default().bg(Color::Blue).fg(Color::Black);
        StatefulWidget::render(
            Table::new(rows, [Constraint::Percentage(100)])
                .header(header)
                .style(help_menu_style)
                .row_highlight_style(selected_style),
            table,
            buf,
            state,
        );

        let config_file = if let Some(path) = ConfigFile::get_config_location() {
            path.to_string_lossy().to_string()
        } else {
            "not found".to_string()
        };
        Paragraph::new(format!(
            "{}\nv {}\n\nconfig:\n{}",
            BANNER,
            env!("CARGO_PKG_VERSION"),
            config_file
        ))
        .alignment(Alignment::Center)
        .render(banner, buf);
    }
}
