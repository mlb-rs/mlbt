use crate::app::{DebugState, MenuItem};
use crate::components::schedule::ScheduleState;
use crate::components::standings::StandingsState;
use crate::components::stats::StatsState;
use crate::state::boxscore::BoxscoreState;
use crate::state::date_input::DateInput;
use crate::state::gameday::GamedayState;

/// A team must be either Home or Away.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum HomeOrAway {
    #[default]
    Home = 0,
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
    pub boxscore_state: BoxscoreState,
    pub standings: StandingsState,
    pub stats: StatsState,
}
