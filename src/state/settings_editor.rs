use crate::components::constants::current_teams_sorted;
use crate::components::standings::Team;
use crate::config::LogLevel;
use crate::state::app_settings::AppSettings;
use chrono_tz::Tz;
use std::sync::LazyLock;

/// Which pane of the help page has input focus.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SettingsFocus {
    #[default]
    Docs,
    Settings,
}

/// A field in the settings form.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    #[default]
    FavoriteTeam,
    Timezone,
    LogLevel,
}

/// Transient status shown below the settings form. Cleared on any input that interacts with the
/// settings editor.
#[derive(Debug, Clone)]
pub enum SettingsStatus {
    Saved,
    Error(String),
}

#[derive(Debug, Default)]
pub struct SettingsEditorState {
    pub focus: SettingsFocus,
    pub selected_field: SettingsField,
    pub picker: Option<PickerState>,
    pub status: Option<SettingsStatus>,
}

/// Picker overlay state that tracks which field is being picked and the cursor position within that
/// field's option list.
#[derive(Debug, Clone)]
pub struct PickerState {
    pub field: SettingsField,
    pub cursor: usize,
}

impl SettingsField {
    pub const ALL: [SettingsField; 3] = [
        SettingsField::FavoriteTeam,
        SettingsField::Timezone,
        SettingsField::LogLevel,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SettingsField::FavoriteTeam => "Team",
            SettingsField::Timezone => "Timezone",
            SettingsField::LogLevel => "Log",
        }
    }

    pub fn next(self) -> Self {
        match self {
            SettingsField::FavoriteTeam => SettingsField::Timezone,
            SettingsField::Timezone => SettingsField::LogLevel,
            SettingsField::LogLevel => SettingsField::FavoriteTeam,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            SettingsField::FavoriteTeam => SettingsField::LogLevel,
            SettingsField::Timezone => SettingsField::FavoriteTeam,
            SettingsField::LogLevel => SettingsField::Timezone,
        }
    }

    pub fn option_count(self) -> usize {
        match self {
            SettingsField::FavoriteTeam => TEAM_OPTIONS.len(),
            SettingsField::Timezone => TIMEZONE_OPTIONS.len(),
            SettingsField::LogLevel => LOG_LEVEL_OPTIONS.len(),
        }
    }

    pub fn option_label(self, index: usize) -> Option<&'static str> {
        match self {
            SettingsField::FavoriteTeam => TEAM_OPTIONS.get(index).map(|o| team_option_label(*o)),
            SettingsField::Timezone => TIMEZONE_OPTIONS.get(index).map(|o| o.picker_label),
            SettingsField::LogLevel => LOG_LEVEL_OPTIONS.get(index).map(|o| o.label),
        }
    }

    /// Index of the current setting value in this field's option list. Falls back to 0 when the
    /// current value is not in the short list (e.g. a hand-edited timezone).
    pub fn current_index(self, settings: &AppSettings) -> usize {
        match self {
            SettingsField::FavoriteTeam => match settings.favorite_team {
                None => 0,
                Some(team) => TEAM_OPTIONS
                    .iter()
                    .position(|o| o.map(|t| t.id) == Some(team.id))
                    .unwrap_or(0),
            },
            SettingsField::Timezone => TIMEZONE_OPTIONS
                .iter()
                .position(|o| o.tz == settings.timezone)
                .unwrap_or(0),
            SettingsField::LogLevel => LOG_LEVEL_OPTIONS
                .iter()
                .position(|o| o.value == settings.log_level)
                .unwrap_or(0),
        }
    }

    /// Apply the option at `index` to `settings`.
    pub fn apply(self, index: usize, settings: &mut AppSettings) {
        match self {
            SettingsField::FavoriteTeam => {
                if let Some(opt) = TEAM_OPTIONS.get(index) {
                    settings.favorite_team = *opt;
                }
            }
            SettingsField::Timezone => {
                if let Some(opt) = TIMEZONE_OPTIONS.get(index) {
                    settings.timezone = opt.tz;
                    settings.refresh_timezone_abbreviation();
                }
            }
            SettingsField::LogLevel => {
                if let Some(opt) = LOG_LEVEL_OPTIONS.get(index) {
                    settings.log_level = opt.value;
                }
            }
        }
    }
}

impl SettingsEditorState {
    pub fn toggle_focus(&mut self) {
        // cannot switch focus away while a picker is open
        if self.picker.is_some() {
            return;
        }
        self.focus = match self.focus {
            SettingsFocus::Docs => SettingsFocus::Settings,
            SettingsFocus::Settings => SettingsFocus::Docs,
        };
        self.status = None;
    }

    pub fn next_field(&mut self) {
        self.selected_field = self.selected_field.next();
        self.status = None;
    }

    pub fn previous_field(&mut self) {
        self.selected_field = self.selected_field.previous();
        self.status = None;
    }

    /// Open the picker for the currently selected field, seeding the cursor at the given option
    /// index (usually the field's current value).
    pub fn open_picker(&mut self, cursor: usize) {
        self.picker = Some(PickerState {
            field: self.selected_field,
            cursor,
        });
        self.status = None;
    }

    pub fn close_picker(&mut self) {
        self.picker = None;
    }

    pub fn picker_next(&mut self) {
        if let Some(p) = &mut self.picker {
            let count = p.field.option_count();
            if count > 0 {
                p.cursor = (p.cursor + 1) % count;
            }
        }
    }

    pub fn picker_previous(&mut self) {
        if let Some(p) = &mut self.picker {
            let count = p.field.option_count();
            if count > 0 {
                p.cursor = if p.cursor == 0 {
                    count - 1
                } else {
                    p.cursor - 1
                };
            }
        }
    }

    /// Move the picker cursor to the next option whose label starts with `c` (case insensitive),
    /// cycling through matches on repeated presses.
    pub fn picker_jump_to_char(&mut self, c: char) {
        let Some(p) = &mut self.picker else { return };
        let count = p.field.option_count();
        if count == 0 {
            return;
        }

        let target = c.to_ascii_lowercase();
        if let Some(idx) = (1..=count).map(|i| (p.cursor + i) % count).find(|&idx| {
            p.field
                .option_label(idx)
                .is_some_and(|s| s.starts_with(|first: char| first.to_ascii_lowercase() == target))
        }) {
            p.cursor = idx;
        }
    }
}

/// Current value of a field rendered as a human-readable string.
pub fn current_value_label(field: SettingsField, settings: &AppSettings) -> String {
    match field {
        SettingsField::FavoriteTeam => settings
            .favorite_team
            .map(|t| t.team_name.to_string())
            .unwrap_or_else(|| "<none>".to_string()),
        SettingsField::Timezone => TIMEZONE_OPTIONS
            .iter()
            .find(|o| o.tz == settings.timezone)
            .map(|o| o.label.to_string())
            .unwrap_or_else(|| settings.timezone.name().to_string()),
        SettingsField::LogLevel => LOG_LEVEL_OPTIONS
            .iter()
            .find(|o| o.value == settings.log_level)
            .map(|o| o.label.to_string())
            .unwrap_or_else(|| "<unset>".to_string()),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TzOption {
    /// Short name shown in the settings row.
    pub label: &'static str,
    /// Verbose label with abbreviations shown in the picker dropdown.
    pub picker_label: &'static str,
    pub tz: Tz,
}

#[rustfmt::skip]
pub const TIMEZONE_OPTIONS: &[TzOption] = &[
    TzOption {label: "Pacific", picker_label: "Pacific (PST/PDT)", tz: chrono_tz::US::Pacific},
    TzOption {label: "Arizona", picker_label: "Arizona (MST)", tz: chrono_tz::US::Arizona},
    TzOption {label: "Mountain", picker_label: "Mountain (MST/MDT)", tz: chrono_tz::US::Mountain},
    TzOption {label: "Central", picker_label: "Central (CST/CDT)", tz: chrono_tz::US::Central},
    TzOption {label: "Eastern", picker_label: "Eastern (EST/EDT)", tz: chrono_tz::US::Eastern},
    TzOption {label: "London", picker_label: "London (GMT/BST)", tz: chrono_tz::Europe::London},
    TzOption {label: "Berlin", picker_label: "Central Europe (CET/CEST)", tz: chrono_tz::Europe::Berlin},
    TzOption {label: "Tokyo", picker_label: "Tokyo (JST)", tz: chrono_tz::Asia::Tokyo},
    TzOption {label: "Seoul", picker_label: "Seoul (KST)", tz: chrono_tz::Asia::Seoul},
    TzOption {label: "Sydney", picker_label: "Sydney (AEST/AEDT)", tz: chrono_tz::Australia::Sydney},
    TzOption {label: "UTC", picker_label: "UTC", tz: chrono_tz::UTC},
];

#[derive(Debug, Clone, Copy)]
pub struct LogLevelOption {
    pub label: &'static str,
    pub value: LogLevel,
}

#[rustfmt::skip]
pub const LOG_LEVEL_OPTIONS: &[LogLevelOption] = &[
    LogLevelOption {label: "Off", value: LogLevel::Off},
    LogLevelOption {label: "Error", value: LogLevel::Error},
    LogLevelOption {label: "Warn", value: LogLevel::Warn},
    LogLevelOption {label: "Info", value: LogLevel::Info},
    LogLevelOption {label: "Debug", value: LogLevel::Debug},
    LogLevelOption {label: "Trace", value: LogLevel::Trace},
];

/// Team picker list: `<none>` sentinel first, then all current MLB teams sorted by name.
pub static TEAM_OPTIONS: LazyLock<Vec<Option<Team>>> = LazyLock::new(|| {
    let mut v: Vec<Option<Team>> = vec![None];
    v.extend(current_teams_sorted().iter().copied().map(Some));
    v
});

pub fn team_option_label(opt: Option<Team>) -> &'static str {
    opt.map(|t| t.name).unwrap_or("<none>")
}

/// Width of the widest value a field can ever display in the settings row.
/// Used to size the settings box so it stays stable across selections.
pub fn max_value_width(field: SettingsField) -> usize {
    match field {
        SettingsField::FavoriteTeam => current_teams_sorted()
            .iter()
            .map(|t| t.team_name.chars().count())
            .chain(std::iter::once("<none>".chars().count()))
            .max()
            .unwrap_or(0),
        SettingsField::Timezone => TIMEZONE_OPTIONS
            .iter()
            .map(|o| o.label.chars().count())
            .max()
            .unwrap_or(0),
        SettingsField::LogLevel => LOG_LEVEL_OPTIONS
            .iter()
            .map(|o| o.label.chars().count())
            .max()
            .unwrap_or(0),
    }
}
