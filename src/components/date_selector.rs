use crate::config::TIMEZONE;
use chrono::{NaiveDate, ParseError, Utc};
use chrono_tz::Tz;

pub struct DateSelector {
    pub timezone: Tz,
    pub date: NaiveDate,
    /// Used for selecting the date with arrow keys.
    pub selection_offset: i64,
}

impl Default for DateSelector {
    fn default() -> Self {
        let date = Utc::now().with_timezone(&TIMEZONE).date_naive();
        Self {
            timezone: TIMEZONE,
            date,
            selection_offset: 0,
        }
    }
}

impl DateSelector {
    pub fn new(date: NaiveDate, timezone: Tz) -> Self {
        Self {
            timezone,
            date,
            selection_offset: 0,
        }
    }

    /// Set the date from the input string from the date picker.
    pub fn set_date_from_input(&mut self, date: String) -> Result<(), ParseError> {
        self.date = match date.as_str() {
            "today" => Utc::now().with_timezone(&self.timezone).date_naive(),
            _ => NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d")?,
        };
        Ok(())
    }

    /// Set the date using Left/Right arrow keys to move a single day at a time.
    pub fn set_date_with_arrows(&mut self, forward: bool) -> NaiveDate {
        match forward {
            true => self.selection_offset += 1,
            false => self.selection_offset -= 1,
        }
        Utc::now().with_timezone(&self.timezone).date_naive()
            + chrono::Duration::days(self.selection_offset)
    }

    /// Format the data to be used in the title of a border;
    pub fn format_date_border_title(&self) -> String {
        self.date.format(" %B %d, %Y ").to_string()
    }
}
