pub mod live;
pub mod schedule;

use crate::live::LiveResponse;
use crate::schedule::ScheduleResponse;

use chrono::NaiveDate;
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
        self.get::<schedule::ScheduleResponse>(url)
    }

    pub fn get_schedule_date(&self, date: NaiveDate) -> ScheduleResponse {
        let url = format!(
            "{}v1/schedule?sportId=1&date={}",
            self.base_url,
            date.format("%Y-%m-%d").to_string()
        );
        self.get::<schedule::ScheduleResponse>(url)
    }

    pub fn get_live_data(&self, game_id: u64) -> LiveResponse {
        if game_id == 0 {
            return LiveResponse::default();
        }
        let url = format!(
            "{}v1.1/game/{}/feed/live?language=en",
            self.base_url, game_id
        );
        self.get::<live::LiveResponse>(url)
    }

    // TODO need better error handling, especially on parsing
    fn get<T: Default + DeserializeOwned>(&self, url: String) -> T {
        let response = self.client.get(url).send();
        let response = match response {
            Ok(r) => r,
            Err(e) => {
                panic!("network error {:?}", e);
            }
        };
        let json = response.json::<T>().map(From::from);
        match json {
            Ok(j) => j,
            Err(e) => {
                eprintln!("parsing error {:?}", e);
                T::default()
            }
        }
    }
}
