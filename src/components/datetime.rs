use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

/// Format a game start time in the configured timezone as "7:05 pm".
pub fn format_game_time(utc: DateTime<Utc>, tz: Tz) -> String {
    utc.with_timezone(&tz).format("%-I:%M %P").to_string()
}

/// Same as `format_game_time`, but pads single-digit hours with a leading space (" 7:05 pm") for
/// table column alignment.
pub fn format_game_time_padded(utc: DateTime<Utc>, tz: Tz) -> String {
    utc.with_timezone(&tz).format("%l:%M %P").to_string()
}

/// Format a date as "Apr 13".
pub fn format_short_date(d: NaiveDate) -> String {
    d.format("%b %-d").to_string()
}

/// Format a date as "4/13/2026".
pub fn format_numeric_date(d: NaiveDate) -> String {
    d.format("%-m/%-d/%Y").to_string()
}

/// Format an optional date as "Apr 13", or `fallback` for `None`.
pub fn format_short_date_or(date: Option<NaiveDate>, fallback: &str) -> String {
    date.map(format_short_date)
        .unwrap_or_else(|| fallback.to_string())
}

/// Format an optional date as "4/13/2026", or `fallback` for `None`.
pub fn format_numeric_date_or(date: Option<NaiveDate>, fallback: &str) -> String {
    date.map(format_numeric_date)
        .unwrap_or_else(|| fallback.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn format_game_time_variants() {
        // Single digit hour: padded version adds a leading space
        let utc = Utc.with_ymd_and_hms(2026, 4, 13, 23, 5, 0).unwrap();
        assert_eq!(format_game_time(utc, chrono_tz::US::Eastern), "7:05 pm");
        assert_eq!(
            format_game_time_padded(utc, chrono_tz::US::Eastern),
            " 7:05 pm"
        );

        // Double digit hour: no padding difference
        let utc = Utc.with_ymd_and_hms(2026, 4, 13, 14, 30, 0).unwrap();
        assert_eq!(
            format_game_time_padded(utc, chrono_tz::US::Eastern),
            "10:30 am"
        );
    }

    #[test]
    fn format_dates() {
        let d = NaiveDate::from_ymd_opt(2026, 4, 13).unwrap();
        assert_eq!(format_short_date(d), "Apr 13");
        assert_eq!(format_numeric_date(d), "4/13/2026");
    }

    #[test]
    fn format_or_handles_missing() {
        let d = NaiveDate::from_ymd_opt(2026, 4, 13).unwrap();
        assert_eq!(format_short_date_or(Some(d), "-"), "Apr 13");
        assert_eq!(format_numeric_date_or(Some(d), "-"), "4/13/2026");
        assert_eq!(format_short_date_or(None, "-"), "-");
        assert_eq!(format_numeric_date_or(None, "---"), "---");
    }
}
