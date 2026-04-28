use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

/// Parse a date in the API's `YYYY-MM-DD` form.
pub fn parse_api_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Parse an RFC 3339 datetime to UTC. Accepts any offset, not just `Z`.
pub fn parse_api_datetime(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Format a game start time in the configured timezone as "7:05 pm".
pub fn format_game_time(utc: DateTime<Utc>, tz: Tz) -> String {
    utc.with_timezone(&tz).format("%-I:%M %P").to_string()
}

/// Same as `format_game_time`, but pads single-digit hours with a leading space (" 7:05 pm") for
/// table column alignment.
pub fn format_game_time_padded(utc: DateTime<Utc>, tz: Tz) -> String {
    utc.with_timezone(&tz).format("%l:%M %P").to_string()
}

/// Format an API-shaped `YYYY-MM-DD` string as "Apr 13". Returns `None` on parse failure.
pub fn format_short_date(s: &str) -> Option<String> {
    parse_api_date(s).map(|d| d.format("%b %-d").to_string())
}

/// Format an API-shaped `YYYY-MM-DD` string as "4/13/2026". Returns `None` on parse failure.
pub fn format_numeric_date(s: &str) -> Option<String> {
    parse_api_date(s).map(|d| d.format("%-m/%-d/%Y").to_string())
}

/// Format an optional `YYYY-MM-DD` string as "Apr 13", or `fallback` for missing or unparseable
/// input.
pub fn format_short_date_or<S: AsRef<str>>(date: Option<&S>, fallback: &str) -> String {
    date.and_then(|s| format_short_date(s.as_ref()))
        .unwrap_or_else(|| fallback.to_string())
}

/// Format an optional `YYYY-MM-DD` string as "4/13/2026", or `fallback` for missing or unparseable
/// input.
pub fn format_numeric_date_or<S: AsRef<str>>(date: Option<&S>, fallback: &str) -> String {
    date.and_then(|s| format_numeric_date(s.as_ref()))
        .unwrap_or_else(|| fallback.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn parse_api_date_valid_and_invalid() {
        assert_eq!(
            parse_api_date("2026-04-13"),
            NaiveDate::from_ymd_opt(2026, 4, 13)
        );
        assert_eq!(parse_api_date("not-a-date"), None);
    }

    #[test]
    fn parse_api_datetime_various_formats() {
        // Z suffix
        let got = parse_api_datetime("2026-04-13T23:05:00Z").unwrap();
        assert_eq!(got, Utc.with_ymd_and_hms(2026, 4, 13, 23, 5, 0).unwrap());

        // Non-UTC offset yields the correct UTC instant
        let got = parse_api_datetime("2026-04-13T19:05:00-04:00").unwrap();
        assert_eq!(got, Utc.with_ymd_and_hms(2026, 4, 13, 23, 5, 0).unwrap());

        // Invalid input
        assert_eq!(parse_api_datetime("nope"), None);
    }

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
        assert_eq!(format_short_date("2026-04-13").as_deref(), Some("Apr 13"));
        assert_eq!(
            format_numeric_date("2026-04-13").as_deref(),
            Some("4/13/2026")
        );

        // Parse failure yields None
        assert_eq!(format_short_date("not-a-date"), None);
        assert_eq!(format_numeric_date(""), None);
    }

    #[test]
    fn format_or_handles_missing_and_unparseable() {
        let valid = String::from("2026-04-13");
        assert_eq!(format_short_date_or(Some(&valid), "-"), "Apr 13");
        assert_eq!(format_numeric_date_or(Some(&valid), "-"), "4/13/2026");

        // Missing input falls back
        assert_eq!(format_short_date_or::<String>(None, "-"), "-");
        assert_eq!(format_numeric_date_or::<String>(None, "---"), "---");

        // Unparseable input also falls back
        let bad = String::from("not-a-date");
        assert_eq!(format_short_date_or(Some(&bad), "-"), "-");
        assert_eq!(format_numeric_date_or(Some(&bad), "---"), "---");
    }
}
