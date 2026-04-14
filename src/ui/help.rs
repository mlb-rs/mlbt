use crate::app::MenuItem;
use crate::components::banner::BANNER;
use crate::config::TomlFileStore;
use crate::state::app_settings::AppSettings;
use crate::state::settings_editor::{
    PickerState, SettingsEditorState, SettingsField, SettingsFocus, SettingsStatus,
    current_value_label, max_value_width,
};
use tui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use tui::prelude::*;
use tui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Row, Table, TableState,
};

const HEADER: &[&str; 2] = &["Description", "Key"];
const GENERAL_DOCS: &[&[&str; 2]; 9] = &[
    &["Exit help", "Esc"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Page down", "Shift + j/↓"],
    &["Page up", "Shift + k/↑"],
    &["Quit", "q"],
    &["Full screen", "f"],
    &["Focus settings", "Tab"],
    &["Change setting", "Enter"],
];
const SCOREBOARD_DOCS: &[&[&str; 2]; 9] = &[
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
const GAMEDAY_DOCS: &[&[&str; 2]; 12] = &[
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
const STATS_DOCS: &[&[&str; 2]; 16] = &[
    &["Stats", "3"],
    &["Switch hitting/pitching", "h/p"],
    &["Switch team/player", "t/l"],
    &["Switch pane", "←/→/Tab"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Page down", "Shift + j/↓"],
    &["Page up", "Shift + k/↑"],
    &["View player/team", "Enter"],
    &["Select date", ":"],
    &["Search", " "],
    &[" Fuzzy search", "Ctrl + f"],
    &["Options", " "],
    &[" Toggle stat", "Enter"],
    &[" Sort by stat", "s"],
    &[" Toggle options pane", "o"],
];
const STANDINGS_DOCS: &[&[&str; 2]; 6] = &[
    &["Standings", "4"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["View team", "Enter"],
    &["Select date", ":"],
    &["Toggle division/league", "l"],
];
const TEAM_PAGE_DOCS: &[&[&str; 2]; 10] = &[
    &["Team Page", " "],
    &[" Switch section", "←/→/Tab"],
    &[" Move down", "j/↓"],
    &[" Move up", "k/↑"],
    &[" Page down", "Shift + j/↓"],
    &[" Page up", "Shift + k/↑"],
    &[" Toggle calendar", "c"],
    &[" Toggle roster type", "r"],
    &[" View player", "Enter"],
    &[" Close team page", "Esc"],
];
const PLAYER_PROFILE_DOCS: &[&[&str; 2]; 7] = &[
    &["Player Profile", " "],
    &[" Toggle category", "s"],
    &[" Scroll down", "j/↓"],
    &[" Scroll up", "k/↑"],
    &[" Page down", "Shift + j/↓"],
    &[" Page up", "Shift + k/↑"],
    &[" Close profile", "Esc"],
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

    pub fn page_down(&mut self) {
        self.state.scroll_down_by(10);
    }

    pub fn page_up(&mut self) {
        self.state.scroll_up_by(10);
    }

    pub fn reset(&mut self) {
        self.state.select(Some(0));
    }
}

pub struct HelpWidget<'a> {
    pub active_tab: MenuItem,
    pub settings: &'a AppSettings,
    pub editor: &'a SettingsEditorState,
}

impl<'a> StatefulWidget for HelpWidget<'a> {
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

        let [table_area, right] = Layout::horizontal([Constraint::Length(50), Constraint::Min(30)])
            .flex(Flex::Legacy)
            .margin(1)
            .horizontal_margin(2)
            .areas(area);

        let selected_style = Style::default().bg(Color::Blue).fg(Color::Black);
        let mut docs_table = Table::new(rows, [Constraint::Percentage(100)])
            .header(header)
            .style(help_menu_style);
        // only show selected row if the docs table is focused.
        if self.editor.focus == SettingsFocus::Docs {
            docs_table = docs_table.row_highlight_style(selected_style);
        }
        StatefulWidget::render(docs_table, table_area, buf, state);

        let [banner_area, settings_area, config_area] = Layout::vertical([
            Constraint::Length(8),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .areas(right);

        Paragraph::new(format!("{}\nv {}", BANNER, env!("CARGO_PKG_VERSION")))
            .alignment(Alignment::Center)
            .render(banner_area, buf);

        render_settings(self.settings, self.editor, settings_area, buf);

        let config_path = TomlFileStore::default_path()
            .map(|p| tilde_path(&p))
            .unwrap_or_else(|| "config: not found".to_string());
        Paragraph::new(config_path)
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::DIM))
            .render(config_area, buf);

        if let Some(picker) = &self.editor.picker {
            render_picker(picker, area, buf);
        }
    }
}

/// Collapse the user's home directory to `~`, using the platform-native separator
/// (e.g. `~/.config/mlbt/mlbt.toml` on Unix, `~\AppData\Roaming\mlbt\mlbt.toml` on Windows).
/// Returns the full path unchanged if the home directory can't be determined or doesn't prefix it.
fn tilde_path(path: &std::path::Path) -> String {
    if let Some(base) = directories::BaseDirs::new()
        && let Ok(rel) = path.strip_prefix(base.home_dir())
    {
        return format!("~{}{}", std::path::MAIN_SEPARATOR, rel.display());
    }
    path.display().to_string()
}

/// Centered popup overlay listing options for the current picker field.
fn render_picker(picker: &PickerState, full_area: Rect, buf: &mut Buffer) {
    let count = picker.field.option_count();
    let items: Vec<ListItem> = (0..count)
        .map(|i| ListItem::new(picker.field.option_label(i).unwrap_or("")))
        .collect();

    let title = match picker.field {
        SettingsField::FavoriteTeam => " Favorite team ",
        SettingsField::Timezone => " Timezone ",
        SettingsField::LogLevel => " Log level ",
    };

    // Size the popup: width wide enough for the longest label + borders + padding, height capped so
    // long lists (teams) scroll rather than overflow the page.
    let max_label = (0..count)
        .filter_map(|i| picker.field.option_label(i))
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0);
    let popup_width = (max_label as u16 + 4)
        .max(title.len() as u16 + 2)
        .min(full_area.width);
    let popup_height = (count as u16 + 2)
        .min(full_area.height.saturating_sub(2))
        .max(5);

    let area = centered_rect(full_area, popup_width, popup_height);

    Clear.render(area, buf);

    let mut list_state = ListState::default();
    list_state.select(Some(picker.cursor));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    StatefulWidget::render(list, area, buf, &mut list_state);
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let [vertical] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let [centered] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(vertical);
    centered
}

fn render_settings(
    settings: &AppSettings,
    editor: &SettingsEditorState,
    area: Rect,
    buf: &mut Buffer,
) {
    let settings_focused = editor.focus == SettingsFocus::Settings;
    // Match the docs table's selected-row highlight.
    let focused_row_style = Style::default().bg(Color::Blue).fg(Color::Black);

    let values: Vec<(SettingsField, String)> = SettingsField::ALL
        .iter()
        .map(|f| (*f, current_value_label(*f, settings)))
        .collect();

    let label_w = values
        .iter()
        .map(|(f, _)| f.label().len())
        .max()
        .unwrap_or(0);
    // Size the value column to the widest possible value across every option list so the box stays
    // a stable width regardless of current selection.
    let value_w = SettingsField::ALL
        .iter()
        .map(|f| max_value_width(*f))
        .max()
        .unwrap_or(0);

    let rows: Vec<Row> = values
        .iter()
        .map(|(field, value)| {
            let is_selected = editor.selected_field == *field;
            let style = if is_selected && settings_focused {
                focused_row_style
            } else {
                Style::default()
            };
            Row::new(vec![field.label().to_string(), value.clone()]).style(style)
        })
        .collect();

    const COLUMN_SPACING: u16 = 2;
    let content_width = label_w as u16 + COLUMN_SPACING + value_w as u16;
    // Add 2 for the left/right borders + 2 for inner padding so the highlight doesn't touch the
    // border glyphs.
    let box_width = (content_width + 4).min(area.width);
    let box_height = (rows.len() as u16 + 2).min(area.height.saturating_sub(2));
    let box_x = area.x + area.width.saturating_sub(box_width) / 2;

    let box_area = Rect {
        x: box_x,
        y: area.y,
        width: box_width,
        height: box_height,
    };

    Widget::render(
        Table::new(
            rows,
            [
                Constraint::Length(label_w as u16),
                Constraint::Length(value_w as u16),
            ],
        )
        .column_spacing(COLUMN_SPACING)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Settings ")
                .padding(tui::widgets::Padding::horizontal(1)),
        ),
        box_area,
        buf,
    );

    // Status line centered below the box. Cleared by any input that interacts with the settings
    // editor (see SettingsEditorState).
    if let Some(status) = &editor.status {
        let (text, color) = match status {
            SettingsStatus::Saved => ("Saved!".to_string(), Color::Green),
            SettingsStatus::Error(msg) => (format!("save failed: {msg}"), Color::Red),
        };
        let status_y = box_area.y + box_area.height + 1;
        if status_y < area.y + area.height {
            Paragraph::new(Span::styled(text, Style::default().fg(color)))
                .alignment(Alignment::Center)
                .render(
                    Rect {
                        x: area.x,
                        y: status_y,
                        width: area.width,
                        height: 1,
                    },
                    buf,
                );
        }
    }
}

/// Build the docs so that the order is: general, active tab, other tabs.
/// Team Page and Player Profile docs are inserted once: after Stats when Stats is active,
/// otherwise after Standings.
fn build_docs(active_tab: MenuItem) -> Vec<&'static [&'static str; 2]> {
    let mut docs = GENERAL_DOCS.to_vec();

    match active_tab {
        MenuItem::Gameday => {
            docs.extend_from_slice(GAMEDAY_DOCS);
            docs.extend_from_slice(SCOREBOARD_DOCS);
            docs.extend_from_slice(STATS_DOCS);
            docs.extend_from_slice(STANDINGS_DOCS);
            docs.extend_from_slice(TEAM_PAGE_DOCS);
            docs.extend_from_slice(PLAYER_PROFILE_DOCS);
        }
        MenuItem::Stats => {
            docs.extend_from_slice(STATS_DOCS);
            docs.extend_from_slice(TEAM_PAGE_DOCS);
            docs.extend_from_slice(PLAYER_PROFILE_DOCS);
            docs.extend_from_slice(SCOREBOARD_DOCS);
            docs.extend_from_slice(GAMEDAY_DOCS);
            docs.extend_from_slice(STANDINGS_DOCS);
        }
        MenuItem::Standings => {
            docs.extend_from_slice(STANDINGS_DOCS);
            docs.extend_from_slice(TEAM_PAGE_DOCS);
            docs.extend_from_slice(PLAYER_PROFILE_DOCS);
            docs.extend_from_slice(SCOREBOARD_DOCS);
            docs.extend_from_slice(GAMEDAY_DOCS);
            docs.extend_from_slice(STATS_DOCS);
        }
        // everything else uses the default order
        _ => {
            docs.extend_from_slice(SCOREBOARD_DOCS);
            docs.extend_from_slice(GAMEDAY_DOCS);
            docs.extend_from_slice(STATS_DOCS);
            docs.extend_from_slice(STANDINGS_DOCS);
            docs.extend_from_slice(TEAM_PAGE_DOCS);
            docs.extend_from_slice(PLAYER_PROFILE_DOCS);
        }
    }

    docs
}
