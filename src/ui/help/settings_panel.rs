use crate::state::app_settings::AppSettings;
use crate::state::settings_editor::{
    PickerState, SettingsEditorState, SettingsField, SettingsFocus, SettingsStatus,
    current_value_label, max_value_width,
};
use crate::ui::help::HIGHLIGHT_STYLE;
use tui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use tui::prelude::*;
use tui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Padding, Paragraph, Row, Table,
};

/// Render the bordered settings box plus the Saved/Error status line beneath it.
pub fn render_settings(
    settings: &AppSettings,
    editor: &SettingsEditorState,
    area: Rect,
    buf: &mut Buffer,
) {
    let settings_focused = editor.focus == SettingsFocus::Settings;

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
                HIGHLIGHT_STYLE
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
                .padding(Padding::horizontal(1)),
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

/// Centered popup overlay listing options for the current picker field.
pub fn render_picker(picker: &PickerState, full_area: Rect, buf: &mut Buffer) {
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
