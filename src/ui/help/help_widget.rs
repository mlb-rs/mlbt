use crate::app::MenuItem;
use crate::components::banner::BANNER;
use crate::components::help::{HEADER, RowType, build_docs, format_row};
use crate::config::TomlFileStore;
use crate::state::app_settings::AppSettings;
use crate::state::settings_editor::{SettingsEditorState, SettingsFocus};
use crate::ui::help::settings_panel::{render_picker, render_settings};
use crate::ui::styling::{dim_style, header_style, selected_style};
use tui::layout::{Constraint, Flex, Layout};
use tui::prelude::*;
use tui::widgets::{Paragraph, Row, Table, TableState};

pub struct HelpWidget<'a> {
    pub active_tab: MenuItem,
    pub settings: &'a AppSettings,
    pub editor: &'a SettingsEditorState,
}

impl StatefulWidget for HelpWidget<'_> {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Create a one-column table to avoid flickering due to non-determinism when
        // resolving constraints on widths of table columns.
        let header_style = header_style();
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

        let mut docs_table = Table::new(rows, [Constraint::Percentage(100)])
            .header(header)
            .style(help_menu_style);
        // only show selected row if the docs table is focused.
        if self.editor.focus == SettingsFocus::Docs {
            docs_table = docs_table.row_highlight_style(selected_style());
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
            .style(dim_style())
            .render(config_area, buf);

        if let Some(picker) = &self.editor.picker {
            render_picker(picker, area, buf);
        }
    }
}

/// Collapse the user's home directory to `~`, using the platform-native separator (e.g.
/// `~/.config/mlbt/mlbt.toml` on Unix, `~\AppData\Roaming\mlbt\mlbt.toml` on Windows). Returns the
/// full path unchanged if the home directory can't be determined or doesn't prefix it.
fn tilde_path(path: &std::path::Path) -> String {
    if let Some(base) = directories::BaseDirs::new()
        && let Ok(rel) = path.strip_prefix(base.home_dir())
    {
        return format!("~{}{}", std::path::MAIN_SEPARATOR, rel.display());
    }
    path.display().to_string()
}
