use log::error;
use tui::style::Color;

pub const DIM_COLOR: Color = Color::DarkGray;
/// Default text color which allows the terminal theme to supply the color.
pub const TEXT_COLOR: Color = Color::Reset;

/// Returns `DIM_COLOR` when the value is zero, otherwise fallback to a specified color or the
/// terminal default.
pub trait DimColor {
    /// Returns `DIM_COLOR` when the value is zero, otherwise `fallback`.
    fn dim_or(&self, fallback: Color) -> Color;
    /// Returns `DIM_COLOR` when the value is zero, otherwise `TEXT_COLOR`.
    /// This is preferred in most cases since it allows the terminal theme to supply the color.
    fn dim_or_default(&self) -> Color;
}

macro_rules! impl_dim_color_int {
    ($($t:ty),*) => {
        $(impl DimColor for $t {
            fn dim_or(&self, fallback: Color) -> Color {
                if *self == 0 { DIM_COLOR } else { fallback }
            }
            fn dim_or_default(&self) -> Color {
                self.dim_or(TEXT_COLOR)
            }
        })*
    };
}
impl_dim_color_int!(u8, u16);

impl DimColor for str {
    fn dim_or(&self, fallback: Color) -> Color {
        if self == "0" { DIM_COLOR } else { fallback }
    }

    fn dim_or_default(&self) -> Color {
        self.dim_or(TEXT_COLOR)
    }
}

/// Display an `Option<T>` as a string, using a default if `None`.
/// e.g. `bio.height.display_or("-")`
pub trait OptionDisplayExt {
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
pub trait OptionMapDisplayExt<T> {
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

/// Surname for compact display. Skips trailing generational suffixes so "Vladimir Guerrero Jr."
/// returns "Guerrero" instead of "Jr."
pub fn last_name(full: &str) -> &str {
    let mut parts = full.rsplitn(3, ' ');
    let tail = parts.next().unwrap_or(full);
    if matches!(tail, "Jr." | "Sr." | "II" | "III" | "IV") {
        parts.next().unwrap_or(tail)
    } else {
        tail
    }
}

/// Color for an ERA stat string. Returns `None` for average range (3.00–4.99) so
/// call sites can fall back to their own contextual color.
pub fn era_color(era: &str) -> Option<Color> {
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

/// Color for an ERA stat string or the default color for average range.
pub fn era_color_or_default(era: &str) -> Color {
    era_color(era).unwrap_or(TEXT_COLOR)
}

/// Color for a batting average stat string. Returns `None` for mid-range averages
/// (.100–.299) so call sites can fall back to their own contextual color.
pub fn avg_color(avg: &str) -> Option<Color> {
    avg.parse::<f64>().ok().and_then(|v| {
        if v == 0.0 {
            Some(DIM_COLOR)
        } else if v >= 0.300 {
            Some(Color::Green)
        } else if v < 0.100 {
            Some(Color::Red)
        } else {
            None
        }
    })
}

/// Color for a batting average stat string or the default color for mid-range averages.
pub fn avg_color_or_default(avg: &str) -> Color {
    avg_color(avg).unwrap_or(TEXT_COLOR)
}

/// Color for a winning-percentage stat string. Always returns a color if the value is a valid f64.
pub fn win_pct_color(pct: &str) -> Option<Color> {
    pct.parse::<f64>().ok().map(|v| {
        if v == 0.0 {
            DIM_COLOR
        } else if v >= 0.500 {
            Color::Green
        } else {
            Color::Red
        }
    })
}

/// Color for a winning-percentage stat string or the default color if the value is invalid.
pub fn win_pct_color_or_default(pct: &str) -> Color {
    win_pct_color(pct).unwrap_or(TEXT_COLOR)
}

/// Convert a string from the API to a Color::Rgb. The string starts out as:
/// "rgba(255, 255, 255, 0.55)".
pub fn convert_color(s: String) -> Color {
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
fn test_last_name() {
    assert_eq!(last_name("Jack Flaherty"), "Flaherty");
    assert_eq!(last_name("J.P. France"), "France");
    assert_eq!(last_name("Vladimir Guerrero Jr."), "Guerrero");
    assert_eq!(last_name("Cal Ripken Jr."), "Ripken");
    assert_eq!(last_name("Ken Griffey Sr."), "Griffey");
    assert_eq!(last_name("Cal Ripken III"), "Ripken");
    assert_eq!(last_name("Robert Person II"), "Person");
    assert_eq!(last_name("Madison"), "Madison");
    assert_eq!(last_name(""), "");
    // suffix-only input falls back to the suffix
    assert_eq!(last_name("Jr."), "Jr.");
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
