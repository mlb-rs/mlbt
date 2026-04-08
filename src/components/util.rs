use chrono::NaiveDate;
use log::error;
use tui::style::Color;

/// Returns `Color::DarkGray` when the value is zero, otherwise the given fallback color.
/// e.g. `self.hits.dim_or(color)`
pub(crate) trait DimColor {
    fn dim_or(&self, fallback: Color) -> Color;
}

macro_rules! impl_dim_color_int {
    ($($t:ty),*) => {
        $(impl DimColor for $t {
            fn dim_or(&self, fallback: Color) -> Color {
                if *self == 0 { Color::DarkGray } else { fallback }
            }
        })*
    };
}
impl_dim_color_int!(u8, u16);

impl DimColor for str {
    fn dim_or(&self, fallback: Color) -> Color {
        if self == "0" {
            Color::DarkGray
        } else {
            fallback
        }
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

/// Color for an ERA stat string. Returns `None` for average range (3.00–4.99) so
/// call sites can fall back to their own contextual color.
pub(crate) fn era_color(era: &str) -> Option<Color> {
    era.parse::<f64>().ok().and_then(|v| {
        if v <= 3.00 {
            Some(Color::Green)
        } else if v >= 5.00 {
            Some(Color::Red)
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
            Some(Color::DarkGray)
        } else if v >= 0.300 {
            Some(Color::Green)
        } else if v < 0.100 {
            Some(Color::Red)
        } else {
            None
        }
    })
}

/// Color for a winning-percentage stat string.
pub(crate) fn win_pct_color(pct: &str) -> Option<Color> {
    pct.parse::<f64>().ok().map(|v| {
        if v == 0.0 {
            Color::DarkGray
        } else if v >= 0.500 {
            Color::Green
        } else {
            Color::Red
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
