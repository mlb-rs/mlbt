use crate::components::stats::StatType;
use crate::state::network::LoadingState;
use chrono::NaiveDate;
use crossterm::event::KeyEvent;
use mlbt_api::live::LiveResponse;
use mlbt_api::schedule::ScheduleResponse;
use mlbt_api::standings::StandingsResponse;
use mlbt_api::stats::StatsResponse;
use mlbt_api::win_probability::WinProbabilityResponse;

#[derive(Debug, Clone)]
pub enum NetworkRequest {
    Initialize,
    Schedule {
        date: NaiveDate,
    },
    GameData {
        game_id: u64,
    },
    Standings {
        date: NaiveDate,
    },
    Stats {
        date: NaiveDate,
        stat_type: StatType,
    },
}

#[derive(Debug)]
pub enum NetworkResponse {
    LoadingStateChanged {
        loading_state: LoadingState,
    },
    ScheduleLoaded {
        schedule: ScheduleResponse,
    },
    GameDataLoaded {
        game: Box<LiveResponse>,
        win_probability: WinProbabilityResponse,
    },
    StandingsLoaded {
        standings: StandingsResponse,
    },
    StatsLoaded {
        stats: StatsResponse,
    },
    Initialized,
    // TODO pass through errors from API
    #[allow(dead_code)]
    Error {
        message: String,
    },
}

#[derive(Debug, Clone)]
pub enum UiEvent {
    KeyPressed(KeyEvent),
    Resize,
    AppStarted,
}
