use tui::prelude::{Color, Style};

pub(crate) mod help_widget;
pub(crate) mod settings_panel;

pub(crate) const HIGHLIGHT_STYLE: Style = Style::new().bg(Color::Blue).fg(Color::Black);
