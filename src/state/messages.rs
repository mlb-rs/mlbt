use crate::components::stats::table::StatType;
use crate::state::network::LoadingState;
use chrono::NaiveDate;
use crossterm::event::KeyEvent;
use mlbt_api::client::StatGroup;
use mlbt_api::live::LiveResponse;
use mlbt_api::player::PeopleResponse;
use mlbt_api::schedule::ScheduleResponse;
use mlbt_api::season::GameType;
use mlbt_api::standings::StandingsResponse;
use mlbt_api::stats::StatsResponse;
use mlbt_api::team::{RosterResponse, RosterType, TransactionsResponse};
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
    PlayerProfile {
        player_id: u64,
        group: StatGroup,
        date: NaiveDate,
        game_type: GameType,
    },
    TeamPage {
        team_id: u16,
        date: NaiveDate,
    },
    TeamRoster {
        team_id: u16,
        season: i32,
        roster_type: RosterType,
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
    PlayerProfileLoaded {
        data: PeopleResponse,
        game_type: GameType,
    },
    TeamPageLoaded {
        team_id: u16,
        date: NaiveDate,
        schedule: ScheduleResponse,
        roster: RosterResponse,
        transactions: TransactionsResponse,
    },
    TeamRosterLoaded {
        team_id: u16,
        roster: RosterResponse,
        roster_type: RosterType,
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
