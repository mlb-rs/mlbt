use crate::app::{DebugState, HomeOrAway, MenuItem};
use crate::components::schedule::ScheduleState;
use crate::components::standings::StandingsState;
use crate::components::stats::StatsState;
use crate::state::date_input::DateInput;
use crate::state::gameday::GamedayState;

#[derive(Default)]
pub struct AppState {
    pub active_tab: MenuItem,
    pub previous_tab: MenuItem,
    pub debug_state: DebugState,
    pub date_input: DateInput,
    pub schedule: ScheduleState,
    pub gameday: GamedayState,
    pub boxscore_tab: HomeOrAway,
    pub standings: StandingsState,
    pub stats: StatsState,
}
