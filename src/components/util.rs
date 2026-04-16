use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use log::error;
use tui::style::Color;

use crate::theme::Theme;

/// Returns `Color::DarkGray` when the value is zero, otherwise the given fallback color.
/// e.g. `self.hits.dim_or(color)`
pub(crate) trait DimColor {
    fn dim_or(&self, fallback: Color) -> Color;
}

macro_rules! impl_dim_color_int {
    ($($t:ty),*) => {
        $(impl DimColor for $t {
            fn dim_or(&self, fallback: Color) -> Color {
                if *self == 0 { Theme::DIMMED } else { fallback }
            }
        })*
    };
}
impl_dim_color_int!(u8, u16);

impl DimColor for str {
    fn dim_or(&self, fallback: Color) -> Color {
        if self == "0" { Theme::DIMMED } else { fallback }
    }
}

/// Display an `Option<T>` as a string, using a default if `None`.
/// e.g. `bio.height.display_or("-")`
pub(crate) trait OptionDisplayExt {
    fn display_or(&self, default: &str) -> String;
}

impl<T: std::fmt::Display> OptionDisplayExt for Option<T> {
    fn display_or(&self, default: &str) -> String {
        self.as_ref()
            .map(|v| v.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}

/// Map an `Option<T>` through a function, then display as a string with a default if `None`.
/// e.g. `bio.weight.map_display_or(|w| format!("{w}lb"), "")`
pub(crate) trait OptionMapDisplayExt<T> {
    fn map_display_or<U: std::fmt::Display, F: FnOnce(&T) -> U>(
        &self,
        f: F,
        default: &str,
    ) -> String;
}

impl<T> OptionMapDisplayExt<T> for Option<T> {
    fn map_display_or<U: std::fmt::Display, F: FnOnce(&T) -> U>(
        &self,
        f: F,
        default: &str,
    ) -> String {
        self.as_ref()
            .map(f)
            .map(|v| v.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}

/// Format "YYYY-MM-DD" as "M/D/YYYY", or return the original string if parsing fails.
pub(crate) fn format_date(s: &str) -> String {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(|d| d.format("%-m/%-d/%Y").to_string())
        .unwrap_or_else(|_| s.to_string())
}

/// Format a UTC game start time for schedule/table display in the user's configured timezone.
pub(crate) fn format_start_time_table(utc: DateTime<Utc>, tz: Tz) -> String {
    utc.with_timezone(&tz).format("%l:%M %P").to_string()
}

/// Format a UTC game start time for compact display in the user's configured timezone.
pub(crate) fn format_start_time_compact(utc: DateTime<Utc>, tz: Tz) -> String {
    utc.with_timezone(&tz).format("%-I:%M %P").to_string()
}

/// Color for an ERA stat string. Returns `None` for average range (3.00–4.99) so
/// call sites can fall back to their own contextual color.
pub(crate) fn era_color(era: &str) -> Option<Color> {
    era.parse::<f64>().ok().and_then(|v| {
        if v <= 2.50 {
            Some(Theme::EXCELLENT)
        } else if v <= 3.00 {
            Some(Theme::GOOD)
        } else if v >= 5.00 {
            Some(Theme::POOR)
        } else if v >= 4.00 {
            Some(Theme::BELOW_AVG)
        } else {
            None
        }
    })
}

/// Color for a batting average stat string. Returns `None` for mid-range averages
/// (.100–.299) so call sites can fall back to their own contextual color.
pub(crate) fn avg_color(avg: &str) -> Option<Color> {
    avg.parse::<f64>().ok().and_then(|v| {
        if v == 0.0 {
            Some(Theme::DIMMED)
        } else if v >= 0.300 {
            Some(Theme::EXCELLENT)
        } else if v >= 0.275 {
            Some(Theme::GOOD)
        } else if v < 0.100 {
            Some(Theme::POOR)
        } else if v < 0.200 {
            Some(Theme::BELOW_AVG)
        } else {
            None
        }
    })
}

/// Color for a winning-percentage stat string.
pub(crate) fn win_pct_color(pct: &str) -> Option<Color> {
    pct.parse::<f64>().ok().map(|v| {
        if v == 0.0 {
            Theme::DIMMED
        } else if v >= 0.600 {
            Theme::EXCELLENT
        } else if v >= 0.500 {
            Theme::GOOD
        } else if v >= 0.400 {
            Theme::BELOW_AVG
        } else {
            Theme::POOR
        }
    })
}

/// Color for an OBP stat string. Returns `None` for average range (.290–.349).
pub(crate) fn obp_color(obp: &str) -> Option<Color> {
    obp.parse::<f64>().ok().and_then(|v| {
        if v == 0.0 {
            Some(Theme::DIMMED)
        } else if v >= 0.380 {
            Some(Theme::EXCELLENT)
        } else if v >= 0.350 {
            Some(Theme::GOOD)
        } else if v < 0.250 {
            Some(Theme::POOR)
        } else if v < 0.290 {
            Some(Theme::BELOW_AVG)
        } else {
            None
        }
    })
}

/// Color for a SLG stat string. Returns `None` for average range (.350–.449).
pub(crate) fn slg_color(slg: &str) -> Option<Color> {
    slg.parse::<f64>().ok().and_then(|v| {
        if v == 0.0 {
            Some(Theme::DIMMED)
        } else if v >= 0.500 {
            Some(Theme::EXCELLENT)
        } else if v >= 0.450 {
            Some(Theme::GOOD)
        } else if v < 0.300 {
            Some(Theme::POOR)
        } else if v < 0.350 {
            Some(Theme::BELOW_AVG)
        } else {
            None
        }
    })
}

/// Color for an OPS stat string. Returns `None` for average range (.680–.799).
pub(crate) fn ops_color(ops: &str) -> Option<Color> {
    ops.parse::<f64>().ok().and_then(|v| {
        if v == 0.0 {
            Some(Theme::DIMMED)
        } else if v >= 0.900 {
            Some(Theme::EXCELLENT)
        } else if v >= 0.800 {
            Some(Theme::GOOD)
        } else if v < 0.600 {
            Some(Theme::POOR)
        } else if v < 0.680 {
            Some(Theme::BELOW_AVG)
        } else {
            None
        }
    })
}

/// Color for a WHIP stat string. Returns `None` for average range (1.11–1.39).
/// Lower WHIP is better — color scale is inverted relative to batting stats.
pub(crate) fn whip_color(whip: &str) -> Option<Color> {
    whip.parse::<f64>().ok().and_then(|v| {
        if v == 0.0 {
            Some(Theme::DIMMED)
        } else if v <= 0.90 {
            Some(Theme::EXCELLENT)
        } else if v <= 1.10 {
            Some(Theme::GOOD)
        } else if v >= 1.60 {
            Some(Theme::POOR)
        } else if v >= 1.40 {
            Some(Theme::BELOW_AVG)
        } else {
            None
        }
    })
}

/// Convert a string from the API to a Color::Rgb. The string starts out as:
/// "rgba(255, 255, 255, 0.55)".
pub(crate) fn convert_color(s: String) -> Color {
    if let Some(s) = s.strip_prefix("rgba(") {
        let c: Vec<&str> = s.split(", ").collect();
        Color::Rgb(
            c[0].parse().unwrap_or(0),
            c[1].parse().unwrap_or(0),
            c[2].parse().unwrap_or(0),
        )
    } else {
        error!("color doesn't start with 'rgba(' {s:?}");
        Color::Rgb(0, 0, 0)
    }
}

#[test]
fn test_color_conversion() {
    let tests = vec![
        ("rgba(0, 0, 0, .55)", Color::Rgb(0, 0, 0)),
        ("rgba(6, 90, 238, .55)", Color::Rgb(6, 90, 238)),
        ("rgba(150, 188, 255, .55)", Color::Rgb(150, 188, 255)),
        ("rgba(214, 41, 52, .55)", Color::Rgb(214, 41, 52)),
        ("rgba(255, 255, 255, 0.55)", Color::Rgb(255, 255, 255)),
    ];
    for t in tests {
        assert_eq!(convert_color(t.0.to_string()), t.1);
    }

    let bad = ("rgba(55, 255, 255, 0.55)", Color::Rgb(255, 255, 255));
    assert_ne!(convert_color(bad.0.to_string()), bad.1);

    let nonsense = ("rgba(-5, 255, 255, 0.55)", Color::Rgb(0, 255, 255));
    assert_eq!(convert_color(nonsense.0.to_string()), nonsense.1);
}

#[cfg(test)]
mod stat_color_tests {
    use super::*;

    #[test]
    fn obp_excellent() { assert_eq!(obp_color(".400"), Some(Theme::EXCELLENT)); }
    #[test]
    fn obp_good() { assert_eq!(obp_color(".355"), Some(Theme::GOOD)); }
    #[test]
    fn obp_average() { assert_eq!(obp_color(".310"), None); }
    #[test]
    fn obp_below() { assert_eq!(obp_color(".270"), Some(Theme::BELOW_AVG)); }
    #[test]
    fn obp_poor() { assert_eq!(obp_color(".240"), Some(Theme::POOR)); }
    #[test]
    fn obp_zero() { assert_eq!(obp_color(".000"), Some(Theme::DIMMED)); }

    #[test]
    fn slg_excellent() { assert_eq!(slg_color(".520"), Some(Theme::EXCELLENT)); }
    #[test]
    fn slg_good() { assert_eq!(slg_color(".460"), Some(Theme::GOOD)); }
    #[test]
    fn slg_average() { assert_eq!(slg_color(".400"), None); }
    #[test]
    fn slg_below() { assert_eq!(slg_color(".320"), Some(Theme::BELOW_AVG)); }
    #[test]
    fn slg_poor() { assert_eq!(slg_color(".280"), Some(Theme::POOR)); }

    #[test]
    fn ops_excellent() { assert_eq!(ops_color(".950"), Some(Theme::EXCELLENT)); }
    #[test]
    fn ops_good() { assert_eq!(ops_color(".820"), Some(Theme::GOOD)); }
    #[test]
    fn ops_average() { assert_eq!(ops_color(".730"), None); }
    #[test]
    fn ops_below() { assert_eq!(ops_color(".640"), Some(Theme::BELOW_AVG)); }
    #[test]
    fn ops_poor() { assert_eq!(ops_color(".550"), Some(Theme::POOR)); }

    #[test]
    fn whip_excellent() { assert_eq!(whip_color("0.85"), Some(Theme::EXCELLENT)); }
    #[test]
    fn whip_good() { assert_eq!(whip_color("1.05"), Some(Theme::GOOD)); }
    #[test]
    fn whip_average() { assert_eq!(whip_color("1.25"), None); }
    #[test]
    fn whip_below() { assert_eq!(whip_color("1.45"), Some(Theme::BELOW_AVG)); }
    #[test]
    fn whip_poor() { assert_eq!(whip_color("1.70"), Some(Theme::POOR)); }
    #[test]
    fn whip_zero() { assert_eq!(whip_color("0.00"), Some(Theme::DIMMED)); }
}
