use crate::live::LiveResponse;
use crate::schedule::ScheduleResponse;
use crate::season::{GameType, SeasonInfo, SeasonsResponse};
use crate::standings::StandingsResponse;
use crate::stats::StatsResponse;
use crate::teams::{SportId, TeamsResponse};
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

impl StatGroup {
    /// The default sort stat for player leaderboards.
    pub fn default_sort_stat(&self) -> &'static str {
        match self {
            StatGroup::Hitting => "plateAppearances",
            StatGroup::Pitching => "inningsPitched",
        }
    }
}

impl MLBApi {
    pub async fn get_todays_schedule(&self) -> ApiResult<ScheduleResponse> {
        let url = format!("{}v1/schedule?sportId=1&hydrate=linescore", self.base_url);
        self.get(url).await
    }

    pub async fn get_schedule_date(&self, date: NaiveDate) -> ApiResult<ScheduleResponse> {
        let url = format!(
            "{}v1/schedule?sportId=1&hydrate=linescore&date={}",
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

    /// Fetch season info from the MLB API for a given year.
    pub async fn get_season_info(&self, year: i32) -> ApiResult<Option<SeasonInfo>> {
        let url = format!("{}v1/seasons/{}?sportId=1", self.base_url, year);
        let resp = self.get::<SeasonsResponse>(url).await?;
        Ok(resp.seasons.into_iter().next())
    }

    pub async fn get_standings(
        &self,
        date: NaiveDate,
        game_type: GameType,
    ) -> ApiResult<StandingsResponse> {
        let url = match game_type {
            GameType::SpringTraining => format!(
                "{}v1/standings?sportId=1&season={}&standingsType=springTraining&leagueId=103,104&hydrate=team",
                self.base_url,
                date.year(),
            ),
            GameType::RegularSeason => format!(
                "{}v1/standings?sportId=1&season={}&date={}&leagueId=103,104&hydrate=team",
                self.base_url,
                date.year(),
                date.format("%Y-%m-%d"),
            ),
        };
        self.get(url).await
    }

    pub async fn get_team_stats(
        &self,
        group: StatGroup,
        game_type: GameType,
    ) -> ApiResult<StatsResponse> {
        let local: DateTime<Local> = Local::now();
        let mut url = format!(
            "{}v1/teams/stats?sportId=1&stats=season&season={}&group={}",
            self.base_url,
            local.year(),
            group
        );
        if game_type == GameType::SpringTraining {
            url.push_str("&gameType=S");
        }
        self.get(url).await
    }

    pub async fn get_team_stats_on_date(
        &self,
        group: StatGroup,
        date: NaiveDate,
        game_type: GameType,
    ) -> ApiResult<StatsResponse> {
        let mut url = format!(
            "{}v1/teams/stats?sportId=1&stats=byDateRange&season={}&endDate={}&group={}",
            self.base_url,
            date.year(),
            date.format("%Y-%m-%d"),
            group
        );
        if game_type == GameType::SpringTraining {
            url.push_str("&gameType=S");
        }
        self.get(url).await
    }

    pub async fn get_player_stats(
        &self,
        group: StatGroup,
        game_type: GameType,
    ) -> ApiResult<StatsResponse> {
        let local: DateTime<Local> = Local::now();
        let sort = group.default_sort_stat();
        let mut url = format!(
            "{}v1/stats?sportId=1&stats=season&season={}&group={}&limit=300&sortStat={}&order=desc",
            self.base_url,
            local.year(),
            group,
            sort
        );
        if game_type == GameType::SpringTraining {
            url.push_str("&gameType=S&playerPool=ALL");
        }
        self.get(url).await
    }

    pub async fn get_player_stats_on_date(
        &self,
        group: StatGroup,
        date: NaiveDate,
        game_type: GameType,
    ) -> ApiResult<StatsResponse> {
        let sort = group.default_sort_stat();
        // Spring training doesn't work well with byDateRange, use season instead.
        let url = match game_type {
            GameType::SpringTraining => format!(
                "{}v1/stats?sportId=1&stats=season&season={}&group={}&limit=300&sortStat={}&order=desc&gameType=S&playerPool=ALL",
                self.base_url,
                date.year(),
                group,
                sort
            ),
            GameType::RegularSeason => format!(
                "{}v1/stats?sportId=1&stats=byDateRange&season={}&endDate={}&group={}&limit=300&sortStat={}&order=desc",
                self.base_url,
                date.year(),
                date.format("%Y-%m-%d"),
                group,
                sort
            ),
        };
        self.get(url).await
    }

    pub async fn get_teams(&self, sport_ids: &[SportId]) -> ApiResult<TeamsResponse> {
        let ids: Vec<String> = sport_ids.iter().map(|id| id.to_string()).collect();
        let url = format!(
            "{}v1/teams?sportIds={}&fields=teams,id,name,division,teamName,abbreviation,sport",
            self.base_url,
            ids.join(",")
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
