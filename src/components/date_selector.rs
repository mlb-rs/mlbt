use chrono::{NaiveDate, Utc};

#[derive(Debug)]
pub struct DateSelector {
    pub date: NaiveDate,
    /// Used for selecting the date with arrow keys.
    pub selection_offset: i64,
    /// Set when the date changes after user input. Consumers should check and clear this after handling.
    pub date_changed: bool,
}

impl Default for DateSelector {
    fn default() -> Self {
        Self {
            date: Utc::now().date_naive(),
            selection_offset: 0,
            date_changed: false,
        }
    }
}

impl DateSelector {
    /// Set the date from the validated input string from the date picker.
    pub fn set_date_from_valid_input(&mut self, date: NaiveDate) {
        self.date_changed = date != self.date;
        self.date = date;
        self.selection_offset = 0;
    }

    /// Set the date using Left/Right arrow keys to move a single day at a time.
    pub fn set_date_with_arrows(&mut self, forward: bool) -> NaiveDate {
        match forward {
            true => self.selection_offset += 1,
            false => self.selection_offset -= 1,
        }
        self.date + chrono::Duration::days(self.selection_offset)
    }

    /// Format the data to be used in the title of a border;
    pub fn format_date_border_title(&self) -> String {
        self.date.format(" %B %d, %Y ").to_string()
    }
}
