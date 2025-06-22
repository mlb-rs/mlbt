use crate::live::LiveResponse;
use crate::schedule::ScheduleResponse;
use crate::standings::StandingsResponse;
use crate::stats::StatsResponse;
use crate::win_probability::WinProbabilityResponse;
use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Datelike, Local, NaiveDate};
use derive_builder::Builder;
use reqwest::Client;
use serde::de::DeserializeOwned;

pub type ApiResult<T> = Result<T, ApiError>;

const BASE_URL: &str = "https://statsapi.mlb.com/api/";

/// MLB API object
#[derive(Builder, Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct MLBApi {
    #[builder(default = "Client::new()")]
    client: Client,
    #[builder(default = "Duration::from_secs(10)")]
    timeout: Duration,
    #[builder(setter(into), default = "String::from(BASE_URL)")]
    base_url: String,
}

#[derive(Debug)]
pub enum ApiError {
    Network(reqwest::Error, String),
    API(reqwest::Error, String),
    Parsing(reqwest::Error, String),
}

impl ApiError {
    pub fn log(&self) -> String {
        match self {
            ApiError::Network(e, url) => format!("Network error for {url}: {e:?}"),
            ApiError::API(e, url) => format!("API error for {url}: {e:?}"),
            ApiError::Parsing(e, url) => format!("Parsing error for {url}: {e:?}"),
        }
    }
}

/// The available stat groups. These are taken from the "meta" endpoint:
/// https://statsapi.mlb.com/api/v1/statGroups
/// I only need to use Hitting and Pitching for now.
#[derive(Clone, Copy, Debug)]
pub enum StatGroup {
    Hitting,
    Pitching,
    // Fielding,
    // Catching,
    // Running,
    // Game,
    // Team,
    // Streak,
}

/// Display the StatGroup in all lowercase.
impl fmt::Display for StatGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StatGroup::Hitting => write!(f, "hitting"),
            StatGroup::Pitching => write!(f, "pitching"),
        }
    }
}

impl MLBApi {
    pub async fn get_todays_schedule(&self) -> ApiResult<ScheduleResponse> {
        let url = format!("{}v1/schedule?sportId=1", self.base_url);
        self.get(url).await
    }

    pub async fn get_schedule_date(&self, date: NaiveDate) -> ApiResult<ScheduleResponse> {
        let url = format!(
            "{}v1/schedule?sportId=1&date={}",
            self.base_url,
            date.format("%Y-%m-%d")
        );
        self.get(url).await
    }

    pub async fn get_live_data(&self, game_id: u64) -> ApiResult<LiveResponse> {
        if game_id == 0 {
            return Ok(LiveResponse::default());
        }
        let url = format!(
            "{}v1.1/game/{}/feed/live?language=en",
            self.base_url, game_id
        );
        self.get(url).await
    }

    pub async fn get_win_probability(&self, game_id: u64) -> ApiResult<WinProbabilityResponse> {
        if game_id == 0 {
            return Ok(WinProbabilityResponse::default());
        }
        let url = format!(
            "{}v1/game/{}/winProbability?fields=homeTeamWinProbability&fields=awayTeamWinProbability&fields=homeTeamWinProbabilityAdded&fields=atBatIndex&fields=about&fields=inning&fields=isTopInning&fields=captivatingIndex&fields=leverageIndex",
            self.base_url, game_id
        );
        self.get(url).await
    }

    pub async fn get_standings(&self, date: NaiveDate) -> ApiResult<StandingsResponse> {
        let url = format!(
            "{}v1/standings?sportId=1&season={}&date={}&leagueId=103,104",
            self.base_url,
            date.year(),
            date.format("%Y-%m-%d"),
        );
        self.get(url).await
    }

    pub async fn get_team_stats(&self, group: StatGroup) -> ApiResult<StatsResponse> {
        let local: DateTime<Local> = Local::now();
        let url = format!(
            "{}v1/teams/stats?sportId=1&stats=season&season={}&group={}",
            self.base_url,
            local.year(),
            group
        );
        self.get(url).await
    }

    pub async fn get_team_stats_on_date(
        &self,
        group: StatGroup,
        date: NaiveDate,
    ) -> ApiResult<StatsResponse> {
        let url = format!(
            "{}v1/teams/stats?sportId=1&stats=byDateRange&season={}&endDate={}&group={}",
            self.base_url,
            date.year(),
            date.format("%Y-%m-%d"),
            group
        );
        self.get(url).await
    }

    pub async fn get_player_stats(&self, group: StatGroup) -> ApiResult<StatsResponse> {
        let local: DateTime<Local> = Local::now();
        let url = format!(
            "{}v1/stats?sportId=1&stats=season&season={}&group={}&limit=300",
            self.base_url,
            local.year(),
            group
        );
        self.get(url).await
    }

    pub async fn get_player_stats_on_date(
        &self,
        group: StatGroup,
        date: NaiveDate,
    ) -> ApiResult<StatsResponse> {
        let url = format!(
            "{}v1/stats?sportId=1&stats=byDateRange&season={}&endDate={}&group={}&limit=300",
            self.base_url,
            date.year(),
            date.format("%Y-%m-%d"),
            group
        );
        self.get(url).await
    }

    async fn get<T: Default + DeserializeOwned>(&self, url: String) -> ApiResult<T> {
        let response = self
            .client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|err| ApiError::Network(err, url.clone()))?;

        let status = response.status();
        match response.error_for_status() {
            Ok(res) => res
                .json::<T>()
                .await
                .map_err(|err| ApiError::Parsing(err, url.clone())),
            // 400-5xx returns errors
            Err(err) => {
                if status.is_client_error() {
                    // just swallow 4xx responses
                    Ok(T::default())
                } else {
                    Err(ApiError::API(err, url.clone()))
                }
            }
        }
    }
}

#[test]
fn test_stat_group_lowercase() {
    assert_eq!("hitting".to_string(), StatGroup::Hitting.to_string());
    assert_eq!("pitching".to_string(), StatGroup::Pitching.to_string());
}
