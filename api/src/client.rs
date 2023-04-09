use std::fmt;

use crate::live::LiveResponse;
use crate::schedule::ScheduleResponse;
use crate::standings::StandingsResponse;
use crate::stats::StatResponse;

use chrono::{DateTime, Datelike, Local, NaiveDate};
use derive_builder::Builder;
use reqwest::Client;
use serde::de::DeserializeOwned;

pub const BASE_URL: &str = "http://statsapi.mlb.com/api/";

/// MLB API object
#[derive(Builder, Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct MLBApi {
    #[builder(default = "Client::new()")]
    client: Client,
    #[builder(setter(into), default = "String::from(BASE_URL)")]
    base_url: String,
}

/// The available stat groups. These are taken from the "meta" endpoint:
/// https://statsapi.mlb.com/api/v1/statGroups
/// I only need to use Hitting and Pitching for now.
#[derive(Clone, Debug)]
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
        write!(f, "{}", format!("{:?}", self).to_lowercase())
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

    pub async fn get_standings(&self) -> StandingsResponse {
        let local: DateTime<Local> = Local::now();
        let url = format!(
            "{}v1/standings?sportId=1&season={}&date={}&leagueId=103,104",
            self.base_url,
            local.year(),
            local.format("%Y-%m-%d"),
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

    pub async fn get_player_stats(&self, group: StatGroup) -> StatResponse {
        let local: DateTime<Local> = Local::now();
        let url = format!(
            "{}v1/stats?stats=season&season={}&group={}",
            self.base_url,
            local.year(),
            group
        );
        self.get(url).await
    }

    async fn get<T: Default + DeserializeOwned>(&self, url: String) -> T {
        let response = self.client.get(url).send().await.expect("network error");
        response
            .json::<T>()
            .await
            .map(From::from)
            .unwrap_or_else(|err| {
                eprintln!("parsing error {:?}", err);
                T::default()
            })
    }
}

#[test]
fn test_stat_group_lowercase() {
    assert_eq!("hitting".to_string(), StatGroup::Hitting.to_string());
    assert_eq!("pitching".to_string(), StatGroup::Pitching.to_string());
}
