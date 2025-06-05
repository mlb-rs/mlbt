use crate::components::stats::StatType;
use crate::network::LoadingState;
use chrono::NaiveDate;
use crossterm::event::KeyEvent;
use mlb_api::live::LiveResponse;
use mlb_api::schedule::ScheduleResponse;
use mlb_api::standings::StandingsResponse;
use mlb_api::stats::StatsResponse;

#[derive(Debug, Clone)]
pub enum NetworkRequest {
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
    },
    StandingsLoaded {
        standings: StandingsResponse,
    },
    StatsLoaded {
        stats: StatsResponse,
    },
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
