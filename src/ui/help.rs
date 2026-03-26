use crate::app::MenuItem;
use crate::components::banner::BANNER;
use crate::config::ConfigFile;
use tui::layout::{Alignment, Constraint, Flex, Layout};
use tui::prelude::*;
use tui::widgets::{Paragraph, Row, Table, TableState};

const HEADER: &[&str; 2] = &["Description", "Key"];
pub const GENERAL_DOCS: &[&[&str; 2]; 5] = &[
    &["Exit help", "Esc"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Quit", "q"],
    &["Full screen", "f"],
];
pub const SCOREBOARD_DOCS: &[&[&str; 2]; 9] = &[
    &["Scoreboard", "1"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["View game in Gameday", "Enter"],
    &["Select date", ":"],
    &["Switch boxscore team", "h/a"],
    &["Scroll boxscore down", "Shift + j/↓"],
    &["Scroll boxscore up", "Shift + k/↑"],
    &["Toggle win probability", "w"],
];
pub const GAMEDAY_DOCS: &[&[&str; 2]; 12] = &[
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
];
pub const STATS_DOCS: &[&[&str; 2]; 20] = &[
    &["Stats", "3"],
    &["Switch hitting/pitching", "h/p"],
    &["Switch team/player", "t/l"],
    &["Switch pane", "←/→/Tab"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Page down (stats only)", "Shift + j/↓"],
    &["Page up (stats only)", "Shift + k/↑"],
    &["Select date", ":"],
    &["Options", " "],
    &[" Toggle stat", "Enter"],
    &[" Sort by stat", "s"],
    &[" Toggle options pane", "o"],
    &["Player Profile", " "],
    &[" Search", "Ctrl + f"],
    &[" View player", "Enter"],
    &[" Hide player", "Esc"],
    &[" Toggle category", "s"],
    &[" Scroll down", "j/↓"],
    &[" Scroll up", "k/↑"],
];
pub const STANDINGS_DOCS: &[&[&str; 2]; 5] = &[
    &["Standings", "4"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Select date", ":"],
    &["Toggle division/league", "l"],
];

#[derive(Clone, Copy, Eq, PartialEq)]
enum RowType {
    Header,
    SubHeader,
    Row,
}

/// Used to keep track of row type for styling.
struct HelpRow {
    row_type: RowType,
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

pub struct HelpWidget {
    pub active_tab: MenuItem,
}

impl StatefulWidget for HelpWidget {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create a one-column table to avoid flickering due to non-determinism when
        // resolving constraints on widths of table columns.
        let format_row = |r: &[&str; 2]| -> HelpRow {
            let row_type = if r[1].parse::<u8>().is_ok() {
                RowType::Header
            } else if r[1] == " " {
                RowType::SubHeader
            } else {
                RowType::Row
            };
            HelpRow {
                row_type,
                text: vec![format!("{:30}{:15}", r[0], r[1])],
            }
        };
        let header_style = Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
        let sub_header_style = Style::default().add_modifier(Modifier::BOLD);
        let help_menu_style = Style::default();

        let header = Row::new(format_row(HEADER).text)
            .height(1)
            .bottom_margin(0)
            .style(header_style);

        let docs = build_docs(self.active_tab);
        let rows = docs
            .iter()
            .map(|d| format_row(d))
            .map(|item| match item.row_type {
                RowType::Header => Row::new(item.text).style(header_style),
                RowType::SubHeader => Row::new(item.text).style(sub_header_style),
                RowType::Row => Row::new(item.text).style(help_menu_style),
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

/// Build the docs so that the order is: general, active tab, other tabs.
fn build_docs(active_tab: MenuItem) -> Vec<&'static [&'static str; 2]> {
    let mut docs = GENERAL_DOCS.to_vec();

    // default order
    let sections = [
        (MenuItem::Scoreboard, SCOREBOARD_DOCS as &[_]),
        (MenuItem::Gameday, GAMEDAY_DOCS as &[_]),
        (MenuItem::Stats, STATS_DOCS as &[_]),
        (MenuItem::Standings, STANDINGS_DOCS as &[_]),
    ];

    // put the active tab docs at the top
    if let Some((_, active_section)) = sections.iter().find(|(tab, _)| *tab == active_tab) {
        docs.extend_from_slice(active_section);
    }

    // add remaining docs
    for (tab, section_docs) in sections {
        if tab != active_tab {
            docs.extend_from_slice(section_docs);
        }
    }

    docs
}
