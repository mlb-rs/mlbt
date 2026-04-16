use crate::app::{DebugState, MenuItem};
use crate::components::schedule::ScheduleState;
use crate::components::standings::StandingsState;
use crate::state::boxscore::BoxscoreState;
use crate::state::date_input::DateInput;
use crate::state::gameday::GamedayState;
use crate::state::help::HelpState;
use crate::state::settings_editor::SettingsEditorState;
use crate::state::stats::StatsState;

/// A team must be either Home or Away.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum HomeOrAway {
    Home = 0,
    #[default]
    Away = 1,
}

#[derive(Default)]
pub struct AppState {
    pub active_tab: MenuItem,
    pub previous_tab: MenuItem,
    pub debug_state: DebugState,
    pub show_logs: bool,
    pub date_input: DateInput,
    pub schedule: ScheduleState,
    pub gameday: GamedayState,
    pub box_score: BoxscoreState,
    pub standings: StandingsState,
    pub stats: StatsState,
    pub help: HelpState,
    pub settings_editor: SettingsEditorState,
}
