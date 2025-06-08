use chrono::{NaiveDate, ParseError, Utc};
use chrono_tz::Tz;

/// Get user input for the date and store whether it's valid.
pub struct DateInput {
    pub is_valid: bool,
    pub text: String,
}

impl DateInput {
    pub fn validate_input(&mut self, tz: Tz) -> Result<NaiveDate, ParseError> {
        let input: String = self.text.drain(..).collect();
        let date = match input.as_str() {
            "t" | "today" => Ok(Utc::now().with_timezone(&tz).date_naive()),
            _ => NaiveDate::parse_from_str(input.as_str(), "%Y-%m-%d"),
        };
        self.is_valid = date.is_ok();
        date
    }
}

impl Default for DateInput {
    fn default() -> Self {
        DateInput {
            is_valid: true,
            text: String::new(),
        }
    }
}
