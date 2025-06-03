use std::fmt;

use crate::live::LiveResponse;
use crate::schedule::ScheduleResponse;
use crate::standings::StandingsResponse;
use crate::stats::StatResponse;

use chrono::{DateTime, Datelike, Local, NaiveDate};
use derive_builder::Builder;
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;

type ApiResult<T> = Result<T, ApiError>;

const BASE_URL: &str = "https://statsapi.mlb.com/api/";

/// MLB API object
#[derive(Builder, Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct MLBApi {
    #[builder(default = "Client::new()")]
    client: Client,
    #[builder(setter(into), default = "String::from(BASE_URL)")]
    base_url: String,
}

#[derive(Debug)]
enum ApiError {
    Network(reqwest::Error),
    Parsing(reqwest::Error),
}

impl ApiError {
    fn log(&self, url: &str) {
        match self {
            ApiError::Network(e) => eprintln!("Network error for {url}: {e:?}"),
            ApiError::Parsing(e) => eprintln!("Parsing error for {url}: {e:?}"),
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
    pub async fn get_todays_schedule(&self) -> ScheduleResponse {
        let url = format!("{}v1/schedule?sportId=1", self.base_url);
        self.get(url).await
    }

    pub async fn get_schedule_date(&self, date: NaiveDate) -> ScheduleResponse {
        let url = format!(
            "{}v1/schedule?sportId=1&date={}",
            self.base_url,
            date.format("%Y-%m-%d")
        );
        self.get(url).await
    }

    pub async fn get_live_data(&self, game_id: u64) -> LiveResponse {
        if game_id == 0 {
            return LiveResponse::default();
        }
        let url = format!(
            "{}v1.1/game/{}/feed/live?language=en",
            self.base_url, game_id
        );
        self.get(url).await
    }

    pub async fn get_standings(&self, date: NaiveDate) -> StandingsResponse {
        let url = format!(
            "{}v1/standings?sportId=1&season={}&date={}&leagueId=103,104",
            self.base_url,
            date.year(),
            date.format("%Y-%m-%d"),
        );
        self.get(url).await
    }

    pub async fn get_team_stats(&self, group: StatGroup) -> StatResponse {
        let local: DateTime<Local> = Local::now();
        let url = format!(
            "{}v1/teams/stats?sportId=1&stats=season&season={}&group={}",
            self.base_url,
            local.year(),
            group
        );
        self.get(url).await
    }

    pub async fn get_team_stats_on_date(&self, group: StatGroup, date: NaiveDate) -> StatResponse {
        let url = format!(
            "{}v1/teams/stats?sportId=1&stats=byDateRange&season={}&endDate={}&group={}",
            self.base_url,
            date.year(),
            date.format("%Y-%m-%d"),
            group
        );
        self.get(url).await
    }

    pub async fn get_player_stats(&self, group: StatGroup) -> StatResponse {
        let local: DateTime<Local> = Local::now();
        let url = format!(
            "{}v1/stats?sportId=1&stats=season&season={}&group={}",
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
    ) -> StatResponse {
        let url = format!(
            "{}v1/stats?sportId=1&stats=byDateRange&season={}&endDate={}&group={}",
            self.base_url,
            date.year(),
            date.format("%Y-%m-%d"),
            group
        );
        self.get(url).await
    }

    async fn get<T: Default + DeserializeOwned>(&self, url: String) -> T {
        match self.try_get(&url).await {
            Ok(data) => data,
            Err(error) => {
                error.log(&url);
                T::default()
            }
        }
    }

    // TODO return errors to caller

    async fn try_get<T: Default + DeserializeOwned>(&self, url: &str) -> ApiResult<T> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(ApiError::Network)?;

        if response.status() == StatusCode::OK {
            response.json::<T>().await.map_err(ApiError::Parsing)
        } else {
            // just swallow non 200 responses
            Ok(T::default())
        }
    }
}

#[test]
fn test_stat_group_lowercase() {
    assert_eq!("hitting".to_string(), StatGroup::Hitting.to_string());
    assert_eq!("pitching".to_string(), StatGroup::Pitching.to_string());
}
