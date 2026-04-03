use crate::app::{DebugState, MenuItem};
use crate::components::schedule::ScheduleState;
use crate::components::standings::StandingsState;
use crate::state::boxscore::BoxscoreState;
use crate::state::date_input::DateInput;
use crate::state::gameday::GamedayState;
use crate::state::stats::StatsState;
use crate::ui::help::HelpState;

/// A team must be either Home or Away.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum HomeOrAway {
    #[default]
    Home = 0,
    Away = 1,
}

pub struct AppState {
    pub active_tab: MenuItem,
    pub previous_tab: MenuItem,
    pub debug_state: DebugState,
    pub show_logs: bool,
    pub show_colors: bool,
    pub date_input: DateInput,
    pub schedule: ScheduleState,
    pub gameday: GamedayState,
    pub box_score: BoxscoreState,
    pub standings: StandingsState,
    pub stats: StatsState,
    pub help: HelpState,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            show_colors: true,
            active_tab: Default::default(),
            previous_tab: Default::default(),
            debug_state: Default::default(),
            show_logs: Default::default(),
            date_input: Default::default(),
            schedule: Default::default(),
            gameday: Default::default(),
            box_score: Default::default(),
            standings: Default::default(),
            stats: Default::default(),
            help: Default::default(),
        }
    }
}
