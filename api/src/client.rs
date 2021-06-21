use crate::live::LiveResponse;
use crate::schedule::ScheduleResponse;
use crate::standings::StandingsResponse;

use chrono::{DateTime, Datelike, Local, NaiveDate};
use derive_builder::Builder;
use reqwest::blocking::Client;
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

impl MLBApi {
    pub fn get_todays_schedule(&self) -> ScheduleResponse {
        let url = format!("{}v1/schedule?sportId=1", self.base_url);
        self.get::<ScheduleResponse>(url)
    }

    pub fn get_schedule_date(&self, date: NaiveDate) -> ScheduleResponse {
        let url = format!(
            "{}v1/schedule?sportId=1&date={}",
            self.base_url,
            date.format("%Y-%m-%d").to_string()
        );
        self.get::<ScheduleResponse>(url)
    }

    pub fn get_live_data(&self, game_id: u64) -> LiveResponse {
        if game_id == 0 {
            return LiveResponse::default();
        }
        let url = format!(
            "{}v1.1/game/{}/feed/live?language=en",
            self.base_url, game_id
        );
        self.get::<LiveResponse>(url)
    }

    pub fn get_standings(&self) -> StandingsResponse {
        let local: DateTime<Local> = Local::now();
        let url = format!(
            "{}v1/standings?sportId=1&season={}&date={}&leagueId=103,104",
            self.base_url,
            local.year().to_string(),
            local.format("%Y-%m-%d").to_string(),
        );
        self.get::<StandingsResponse>(url)
    }

    // TODO need better error handling, especially on parsing
    fn get<T: Default + DeserializeOwned>(&self, url: String) -> T {
        let response = self.client.get(url).send().unwrap_or_else(|err| {
            panic!("network error {:?}", err);
        });
        response.json::<T>().map(From::from).unwrap_or_else(|err| {
            eprintln!("parsing error {:?}", err);
            T::default()
        })
    }
}
