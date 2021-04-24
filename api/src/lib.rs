pub mod schedule;

use derive_builder::Builder;
use reqwest::blocking::Client;

use crate::schedule::ScheduleResponse;
use chrono::NaiveDate;

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
    // TODO handle errors
    pub fn get_todays_schedule(&self) -> Result<ScheduleResponse, reqwest::Error> {
        let url = format!("{}v1/schedule?sportId=1", self.base_url);
        let res = self.client.get(url).send()?;
        let json: schedule::ScheduleResponse = res.json()?;
        Ok(json)
    }

    pub fn get_schedule_date(&self, date: NaiveDate) -> Result<ScheduleResponse, reqwest::Error> {
        let url = format!(
            "{}v1/schedule?sportId=1&date={}",
            self.base_url,
            date.format("%Y-%m-%d").to_string()
        );
        let res = self.client.get(url).send()?;
        // println!("Status: {}", res.status());
        let json: schedule::ScheduleResponse = res.json()?;
        Ok(json)
    }
}
